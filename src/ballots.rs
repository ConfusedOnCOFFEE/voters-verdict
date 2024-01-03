#[cfg(feature = "db")]
use crate::db::common::Query;
use crate::{
    common::{
        from_optional_str, Ballot, Candidate, CastBallots, Criterion, Empty, Fill, IdGenerator,
        KnownBallots, QueryableExt, Selfaware, Table as VVTable, Vote, VoteKind, Voting,
    },
    error::VoteErrorKind,
    persistence::ToPersistence,
    routes::API_BALLOTS,
    validator::{compare_pattern_file_names, validate},
};
use chrono::prelude::*;
#[cfg(feature = "diesel_sqlite")]
use diesel::{prelude::*, table};
use rocket::{
    debug, error, get,
    http::Status,
    post,
    request::{FromRequest, Outcome},
    response::status::{Created, Unauthorized},
    serde::{json::Json, Deserialize, Serialize},
    Request,
};

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Voter<'r>(pub &'r str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Voter<'r> {
    type Error = VoteErrorKind<'r>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let default_voting = Voting::empty();
        let voting: Voting = match req.uri().path().segments().get(3) {
            Some(v) => Voting::fill(v, false, "voting").await,
            None => default_voting,
        };
        fn is_correct_invite_code(key: &str, voting: &Voting) -> bool {
            key == voting.invite_code
        }
        let unautorized_error =
            VoteErrorKind::Unauthorized(Unauthorized(String::from("Supply an invite code.")));
        let invite_code_header_value = req.headers().get_one("x-concafe-invite-code");
        if invite_code_header_value.is_some() {
            let has_correct_invite_code =
                is_correct_invite_code(invite_code_header_value.unwrap(), &voting);
            if has_correct_invite_code {
                let user_header_value = req.headers().get_one("x-concafe-user");
                if user_header_value.is_some() {
                    Outcome::Success(Voter(user_header_value.unwrap()))
                } else {
                    Outcome::Error((Status::Unauthorized, unautorized_error))
                }
            } else {
                Outcome::Error((Status::Unauthorized, unautorized_error))
            }
        } else {
            Outcome::Error((Status::Unauthorized, unautorized_error))
        }
    }
}

impl<'a> From<Voter<'a>> for Candidate {
    fn from(v: Voter) -> Self {
        Self {
            voter: true,
            id: Some(v.0.to_string()),
            label: Candidate::get_label_from_id(v.0),
        }
    }
}

#[get("/<voting>/ballots")]
pub async fn get_ballots_by_voting(voting: &str) -> Json<CastBallots> {
    CastBallots::fill_json(voting, false, "CBallots").await
}

#[post("/<voting_id>", format = "application/json", data = "<ballot>")]
pub async fn post_ballot<'r>(
    voter: Voter<'r>,
    voting_id: &str,
    ballot: Json<Ballot>,
) -> Result<Created<&'static str>, Status> {
    if validate(
        VoteKind::Ballot(ballot.clone().into_inner()),
        voting_id,
        voter.0,
    )
    .await
    .is_ok()
    {
        let mut inner_ballot = ballot.into_inner();
        inner_ballot.voted_on = Some(Utc::now());
        let cast_ballot = CastBallots {
            voting: Some(String::from(voting_id)),
            ballots: vec![KnownBallots {
                voter: voter.0.to_string(),
                ballots: vec![inner_ballot],
            }],
        };

        #[cfg(not(feature = "sqlx_sqlite"))]
        match cast_ballot.save().await {
            Ok(done) => Ok(Created::new(
                API_BALLOTS.to_owned() + "/" + voting_id + "/voter/" + voter.0,
            )),
            Err(_e) => Err(Status::UnprocessableEntity),
        }
        #[cfg(feature = "sqlx_sqlite")]
        match BallotsTable::from(CompleteBallotsTable::aggregate_all(cast_ballot).await)
            .save()
            .await
        {
            Ok(_done) => Ok(Created::new(
                API_BALLOTS.to_owned() + "/" + voting_id + "/voter/" + voter.0,
            )),
            Err(_e) => Err(Status::UnprocessableEntity),
        }
    } else {
        Err(Status::UnprocessableEntity)
    }
}
#[cfg(feature = "db")]
impl CompleteBallotsTable {
    async fn aggregate_all(c_B: CastBallots) -> Self {
        let ballots = c_B.ballots.first().unwrap().ballots.first().unwrap();
        let voter = &c_B.ballots.first().unwrap().voter;
        let votes = &ballots.votes;
        let sum = sum_up_sum(&votes.to_vec());
        let voting = Voting::fill(&c_B.voting.clone().unwrap(), false, "voting").await;
        let weighted = sum_up_weight(&votes.to_vec(), &voting.categories);
        let mean = f32::from(sum) / votes.len() as f32;
        Self {
            categories: voting.categories,
            voter: voter.to_string(),
            voting: c_B.voting.unwrap(),
            candidate: String::from(ballots.candidate.as_str()),
            sum: i16::from(sum),
            weighted: weighted,
            mean: mean,
            notes: match ballots.notes.clone() {
                Some(n) => n,
                None => String::from("n/a"),
            },
            votes: match rocket::serde::json::to_string(&ballots.votes) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    String::new()
                }
            },
            voted_on: ballots.voted_on.unwrap(),
        }
    }
}
#[cfg(feature = "db")]
impl From<CompleteBallotsTable> for BallotsTable {
    fn from(cB: CompleteBallotsTable) -> Self {
        Self {
            voter: cB.voter,
            voting: cB.voting,
            candidate: cB.candidate,
            sum: cB.sum,
            weighted: cB.weighted,
            mean: cB.mean,
            notes: cB.notes,
            votes: cB.votes,
            voted_on: cB.voted_on,
        }
    }
}

pub struct TableRowSum {
    sum: i8,
    weighted: f32,
    mean: f32,
    candidate: String,
    notes: String,
    votes: Vec<Vote>,
    voted_on: DateTime<Utc>,
}

#[derive(Serialize, Debug, PartialEq, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TableRow {
    pub voting: String,
    pub voter: String,
    pub candidate: String,
    pub sum: i8,
    pub weighted: f32,
    pub mean: f32,
    notes: String,
    votes: Vec<Vote>,
    pub voted_on: DateTime<Utc>,
}

#[cfg(not(feature = "diesel_sqlite"))]
#[derive(Serialize, Debug, PartialEq, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct BallotsTable {
    pub voting: String,
    pub voter: String,
    pub candidate: String,
    pub sum: i16,
    pub weighted: f32,
    pub mean: f32,
    notes: String,
    votes: String,
    pub voted_on: DateTime<Utc>,
}
#[derive(Serialize, Debug, PartialEq, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CompleteBallotsTable {
    pub categories: Vec<Criterion>,
    pub voting: String,
    pub voter: String,
    pub candidate: String,
    pub sum: i16,
    pub weighted: f32,
    pub mean: f32,
    notes: String,
    votes: String,
    pub voted_on: DateTime<Utc>,
}
impl BallotsTable {
    fn values(&self) -> String {
        format!(
            "'{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}'",
            self.get_id(),
            self.candidate,
            self.voter,
            self.sum,
            self.weighted,
            self.mean,
            self.notes,
            self.votes,
            self.voted_on.to_string()
        )
    }
}
impl Query for BallotsTable {}
impl QueryableExt for BallotsTable {}
impl std::str::FromStr for BallotsTable {
    type Err = crate::error::FromErrorKind;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        debug!("{:?}", v);
        let result: Vec<&str> = v.split(",-o-,").collect();
        debug!("{:?}", result);
        match CastBallots::from_str(v) {
            Ok(_c_b) => Ok(Self {
                voter: from_optional_str(result.get(2).copied()),
                voting: result
                    .get(0)
                    .copied()
                    .expect("The structure is always 0-1-2")
                    .split_once('-')
                    .unwrap()
                    .0
                    .to_string(),
                candidate: from_optional_str(result.get(1).copied()),
                sum: i16::from_str(&from_optional_str(result.get(3).copied())).unwrap(),
                weighted: f32::from_str(&from_optional_str(result.get(4).copied())).unwrap(),
                mean: f32::from_str(&from_optional_str(result.get(5).copied())).unwrap(),
                notes: from_optional_str(result.get(6).copied()),
                votes: from_optional_str(result.get(7).copied()),
                voted_on: DateTime::from_str(&from_optional_str(result.get(8).copied())).unwrap(),
            }),
            Err(_) => Ok(Self::empty()),
        }
    }
}
#[cfg(feature = "sqlx_sqlite")]
impl VVTable for BallotsTable {
    fn get_identity_column_name() -> String {
        String::from("human_identifier")
    }
    fn get_table(insert: bool) -> String {
        if insert {
            String::from("ballots") + &CastBallots::properties(true)
        } else {
            String::from("ballots")
        }
    }
    fn get_db_columns() -> String {
        CastBallots::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}
impl Empty for BallotsTable {
    fn empty() -> Self {
        Self {
            voting: String::new(),
            voter: String::new(),
            candidate: String::new(),
            sum: 0,
            weighted: 0.0,
            mean: 0.0,
            notes: String::new(),
            votes: String::new(),
            voted_on: Utc::now(),
        }
    }
}

impl IdGenerator for BallotsTable {
    fn get_id(&self) -> String {
        self.generate_id()
    }
    fn generate_id(&self) -> String {
        self.voting.to_lowercase()
            + "-"
            + &self.voter.to_lowercase()
            + "-"
            + &self.candidate.to_lowercase()
    }
}
impl Fill for BallotsTable {}
impl crate::serialize::FromStorage for BallotsTable {}
impl crate::persistence::FromPersistence for BallotsTable {}
impl crate::persistence::Path for BallotsTable {}
impl Selfaware for BallotsTable {
    fn get_type(&self) -> String {
        String::from("CBallots")
    }

    fn get_name(&self) -> String {
        String::from("CBallots")
    }
}

#[cfg(feature = "diesel_sqlite")]
#[derive(Serialize, Debug, PartialEq, Clone, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = ballots )]
#[serde(crate = "rocket::serde")]
pub struct BallotsTable {
    pub voter: String,
    pub candidate: String,
    pub sum: i16,
    pub weighted: f32,
    pub mean: f32,
    notes: String,
    votes: String,
    pub voted_on: DateTime<Utc>,
}
#[cfg(feature = "db")]
impl From<TableRow> for BallotsTable {
    fn from(tr: TableRow) -> Self {
        Self {
            voting: tr.voting,
            voter: tr.voter,
            candidate: tr.candidate,
            sum: i16::from(tr.sum),
            weighted: tr.weighted,
            mean: tr.mean,
            notes: tr.notes,
            votes: match rocket::serde::json::to_string(&tr.votes) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    String::new()
                }
            },
            voted_on: tr.voted_on,
        }
    }
}

#[cfg(feature = "db")]
impl From<&BallotsTable> for TableRow {
    fn from(b_t: &BallotsTable) -> Self {
        Self {
            voting: b_t.voting.clone(),
            voter: b_t.voter.clone(),
            candidate: b_t.candidate.clone(),
            sum: i8::try_from(b_t.sum).unwrap(),
            weighted: b_t.weighted,
            mean: b_t.mean,
            notes: b_t.notes.clone(),
            votes: match rocket::serde::json::from_str::<Vec<Vote>>(&b_t.votes) {
                Ok(vs) => vs,
                Err(e) => {
                    error!("From string votes to struct votes didnt work. {:?}", e);
                    vec![]
                }
            },
            voted_on: b_t.voted_on,
        }
    }
}
#[cfg(feature = "file")]
impl TableRow {
    fn from_cast_ballots(
        cast_ballots: Vec<CastBallots>,
        categories: &Vec<Criterion>,
    ) -> Vec<TableRow> {
        match map_ballots(cast_ballots, categories) {
            Some(rows) => rows,
            None => vec![],
        }
    }
}
#[cfg(feature = "file")]
fn map_ballots(ballots: Vec<CastBallots>, categories: &Vec<Criterion>) -> Option<Vec<TableRow>> {
    ballots
        .iter()
        .map(|c| {
            match c
                .ballots
                .iter()
                .map(|b| map_ballot(b.clone(), categories, &c.voting.clone().unwrap()))
                .collect::<Vec<Vec<TableRow>>>()
                .into_iter()
                .reduce(|c, n| [c, n].concat())
            {
                Some(r) => r,
                None => vec![],
            }
        })
        .reduce(|c, n| [c, n].concat())
}
#[cfg(feature = "file")]
fn map_ballot(
    known_ballot: KnownBallots,
    categories: &Vec<Criterion>,
    voting: &str,
) -> Vec<TableRow> {
    known_ballot
        .ballots
        .iter()
        .map(|c| {
            let sum = sum_up_sum(&c.votes);
            let weighted = sum_up_weight(&c.votes, categories);
            let mean = f32::from(sum) / c.votes.len() as f32;
            let notes = match &c.notes {
                Some(n) => String::from(n.as_str()),
                None => String::new(),
            };
            TableRowSum {
                candidate: String::from(c.candidate.as_str()),
                votes: c.votes.to_vec(),
                notes,
                voted_on: c.voted_on.unwrap(),
                sum,
                weighted,
                mean,
            }
        })
        .map(|v| TableRow {
            voting: voting.to_string(),
            voter: String::from(known_ballot.voter.as_str()),
            votes: v.votes,
            candidate: v.candidate,
            notes: v.notes,
            sum: v.sum,
            weighted: v.weighted,
            mean: v.mean,
            voted_on: v.voted_on,
        })
        .collect::<Vec<TableRow>>()
}

fn sum_up_sum(given_votes: &Vec<Vote>) -> i8 {
    given_votes
        .iter()
        .map(|b| b.point)
        .reduce(|b, c| c + b)
        .unwrap()
}
fn sum_up_weight(given_votes: &Vec<Vote>, categories: &Vec<Criterion>) -> f32 {
    given_votes
        .iter()
        .map(|vote| {
            let category = categories.iter().find(|c| vote.name == c.name).unwrap();
            let weight_to_use = match category.weight {
                Some(w) => w * 0.1,
                None => 1.0,
            };
            f32::from(vote.point) * weight_to_use
        })
        .reduce(|acc, next| acc + next)
        .unwrap()
}

#[derive(Serialize, Debug, PartialEq, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Table<H, R> {
    pub headers: Vec<H>,
    pub rows: Vec<R>,
}
impl<H, R> Table<H, R> {
    fn new(headers: Vec<H>, rows: Vec<R>) -> Self {
        Self { headers, rows }
    }
}

fn verify_correct_voting_id(
    voting: Voting,
    voting_id: &str,
    ballots: Vec<TableRow>,
) -> Table<Criterion, TableRow> {
    if voting.name.to_lowercase() == voting_id {
        Table::<Criterion, TableRow>::new(voting.categories, ballots)
    } else {
        Table::<Criterion, TableRow>::new(vec![], vec![])
    }
}

async fn collect_ballots(voting_id: &str) -> Table<Criterion, TableRow> {
    let lowercased_voting_id = voting_id.to_lowercase();
    let loaded_ballots = CastBallots::empty().list().await.unwrap();
    #[cfg(not(feature = "sqlx_sqlite"))]
    let filtered_ballots: Vec<_> = loaded_ballots
        .iter()
        .filter(|b| b.starts_with(&lowercased_voting_id))
        .map(|b| CastBallots::fill(b, true, "CBallots"))
        .collect();
    #[cfg(feature = "sqlx_sqlite")]
    let filtered_ballots: Vec<_> = loaded_ballots
        .iter()
        .filter(|b| b.starts_with(&lowercased_voting_id))
        .map(|b| BallotsTable::fill(b, true, "CBallots"))
        .collect();
    let collected_ballots: Vec<_> = futures::future::join_all(filtered_ballots).await;
    let voting = Voting::fill(&lowercased_voting_id, false, "voting").await;
    debug!("{:?}", collected_ballots);
    #[cfg(not(feature = "sqlx_sqlite"))]
    let ballots = TableRow::from_cast_ballots(collected_ballots, &voting.categories);
    #[cfg(feature = "sqlx_sqlite")]
    let ballots = collected_ballots
        .iter()
        .map(|b| TableRow::from(b))
        .collect();
    verify_correct_voting_id(voting, &lowercased_voting_id, ballots)
}

#[get("/<voting_id>")]
pub async fn get_ballots_by_voted_on(voting_id: &str) -> Json<Table<Criterion, TableRow>> {
    Json(ballots_by_voted_on(voting_id).await)
}
pub async fn ballots_by_voted_on(voting_id: &str) -> Table<Criterion, TableRow> {
    let mut table = collect_ballots(voting_id).await;
    if !table.rows.is_empty() {
        table.rows.sort_by(|p, n| p.voted_on.cmp(&n.voted_on));
    }
    table
}

#[get("/<voting_id>/results?<sort>")]
pub async fn get_ballots_sorted(voting_id: &str, sort: &str) -> Json<Table<Criterion, TableRow>> {
    Json(ballots_sorted(voting_id, sort).await)
}
pub async fn ballots_sorted(voting_id: &str, sort: &str) -> Table<Criterion, TableRow> {
    let mut table = collect_ballots(voting_id).await;
    if !table.rows.is_empty() {
        table.rows.sort_by(|p, n| match sort {
            "sum" => p.sum.cmp(&n.sum),
            "mean" => p.mean.total_cmp(&n.mean),
            "weight" => p.weighted.total_cmp(&n.weighted),
            _ => p.sum.cmp(&n.sum),
        });
    }
    table
}

#[get("/<voting_id>/voters/<voter>")]
pub async fn get_ballots_by_voter(
    voting_id: &str,
    voter: &str,
) -> Json<Table<Criterion, TableRow>> {
    Json(ballots_by_voter(voting_id, voter).await)
}
pub async fn ballots_by_voter(voting_id: &str, voter: &str) -> Table<Criterion, TableRow> {
    let voting = Voting::fill(voting_id, false, "voting").await;
    let ballots: Vec<TableRow> = collect_ballots(voting_id)
        .await
        .rows
        .iter()
        .filter(|b| compare_pattern_file_names(&b.voter, voter))
        .cloned()
        .collect();
    Table::<Criterion, TableRow>::new(voting.categories, ballots)
}

#[get("/<voting_id>/candidates/<candidate>")]
pub async fn get_ballots_by_candidate(
    voting_id: &str,
    candidate: &str,
) -> Json<Table<Criterion, TableRow>> {
    Json(ballots_by_candidate(voting_id, candidate).await)
}
pub async fn ballots_by_candidate(voting_id: &str, candidate: &str) -> Table<Criterion, TableRow> {
    let voting = Voting::fill(voting_id, false, "voting").await;
    let ballots: Vec<TableRow> = collect_ballots(voting_id)
        .await
        .rows
        .iter()
        .filter(|b| compare_pattern_file_names(&b.candidate, candidate))
        .cloned()
        .collect();
    Table::<Criterion, TableRow>::new(voting.categories, ballots)
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::common::VotingStyles;
    #[rocket::async_test]
    async fn ballots_by_candidate_not_available() {
        let response = super::ballots_by_candidate("voting", "candidate_Joe").await;
        assert_eq!(response.headers.len(), 2);
        assert_eq!(response.rows.len(), 0);
    }

    #[rocket::async_test]
    async fn ballots_by_candidate() {
        let response = super::ballots_by_candidate("voting", "joe").await;
        assert_eq!(response.headers.len(), 2);
        assert_eq!(response.rows.len(), 2);
    }

    #[rocket::async_test]
    async fn ballots_by_voter() {
        let response = super::ballots_by_voter("voting", "obama").await;
        assert_eq!(response.headers.len(), 2);
        assert_eq!(response.rows.len(), 2);
        assert_eq!(response.rows.get(1).unwrap().sum, 12);
    }

    #[rocket::async_test]
    async fn ballots_sorted() {
        let response = super::ballots_sorted("voting", "sum").await;
        assert_eq!(response.headers.len(), 2);
        assert_eq!(response.rows.len(), 3);
        assert_eq!(response.rows.get(1).unwrap().sum, 14);
    }

    #[rocket::async_test]
    async fn collect_ballots() {
        let response = super::collect_ballots("voting").await;
        assert_eq!(response.headers.len(), 2);
        assert_eq!(response.rows.len(), 3);
    }

    #[rocket::async_test]
    async fn ballots_by_voted_on() {
        let response = super::ballots_by_voted_on("voting").await;
        assert_eq!(response.headers.len(), 2);
        assert_eq!(response.rows.len(), 3);
        assert_eq!(response.rows.get(1).unwrap().sum, 12);
    }

    #[test]
    fn verify_correct_voting_id() {
        let voting = Voting {
            name: String::from("Voting"),
            expires_at: None,
            created_at: None,
            candidates: vec![],
            categories: vec![],
            styles: VotingStyles::default(),
            invite_code: String::from("T1234"),
        };
        let response = super::verify_correct_voting_id(voting, "voting", vec![]);
        assert_eq!(response.headers.len(), 0);
        assert_eq!(response.rows.len(), 0);
    }

    #[test]
    fn verify_correct_voting_id_with_rows() {
        let voting = Voting {
            name: String::from("Voting"),
            expires_at: None,
            created_at: None,
            candidates: vec![],
            categories: vec![],
            styles: VotingStyles::default(),
            invite_code: String::from("T1234"),
        };
        let rows = vec![TableRow {
            voter: String::from("test"),
            candidate: String::from("test"),
            sum: 9,
            weighted: 9.0,
            notes: String::from("note"),
            mean: 1.0,
            votes: vec![],
            voted_on: chrono::Utc::now(),
        }];
        let response = super::verify_correct_voting_id(voting, "voting", rows);
        assert_eq!(response.headers.len(), 0);
        assert_eq!(response.rows.len(), 1);
    }
}
