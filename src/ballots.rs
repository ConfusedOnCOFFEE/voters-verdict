use chrono::prelude::*;
use rocket::{
    get,
    http::Status,
    post,
    request::{FromRequest, Outcome},
    response::status::Unauthorized,
    serde::{json::Json, Deserialize, Serialize},
    Request,
};

use crate::{
    common::{
        Ballot, Candidate, CastBallots, Criterion, Empty, Fill, Index, KnownBallots, ToJsonFile,
        Vote, VoteErrorKind, VoteKind, Voting,
    },
    validator::{compare_styles_file_names, validate},
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
            Some(v) => Voting::fill(v, false).await,
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
    CastBallots::fill_json(voting, false).await
}

#[post("/<voting_id>", format = "application/json", data = "<ballot>")]
pub async fn post_ballot<'r>(
    voter: Voter<'r>,
    voting_id: &str,
    ballot: Json<Ballot>,
) -> Result<String, Status> {
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
        match cast_ballot.to_json_file().await {
            Ok(done) => Ok(done),
            Err(_e) => Err(Status::UnprocessableEntity),
        }
    } else {
        Err(Status::UnprocessableEntity)
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
    pub voter: String,
    pub candidate: String,
    pub sum: i8,
    pub weighted: f32,
    pub mean: f32,
    notes: String,
    votes: Vec<Vote>,
    pub voted_on: DateTime<Utc>,
}

// TODO Requires split and refactor
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

fn map_ballots(ballots: Vec<CastBallots>, categories: &Vec<Criterion>) -> Option<Vec<TableRow>> {
    ballots
        .iter()
        .map(|c| {
            match c
                .ballots
                .iter()
                .map(|b| map_ballot(b.clone(), categories))
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
fn map_ballot(known_ballot: KnownBallots, categories: &Vec<Criterion>) -> Vec<TableRow> {
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
                Some(w) => f32::from(w) * 0.1,
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
    let filtered_ballots: Vec<_> = loaded_ballots
        .iter()
        .filter(|b| b.starts_with(&lowercased_voting_id))
        .map(|b| CastBallots::fill(b, true))
        .collect();
    let collected_ballots: Vec<CastBallots> = futures::future::join_all(filtered_ballots).await;
    let voting = Voting::fill(&lowercased_voting_id, false).await;
    let ballots = TableRow::from_cast_ballots(collected_ballots, &voting.categories);
    verify_correct_voting_id(voting, &lowercased_voting_id, ballots)
}

#[get("/<voting_id>")]
pub async fn get_ballots_by_voted_on(voting_id: &str) -> Json<Table<Criterion, TableRow>> {
    Json(ballots_by_voted_on(voting_id).await)
}
pub async fn ballots_by_voted_on(voting_id: &str) -> Table<Criterion, TableRow> {
    let mut table = collect_ballots(voting_id).await;
    if table.rows.len() != 0 {
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
    if table.rows.len() != 0 {
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
    let voting = Voting::fill(voting_id, false).await;
    let ballots: Vec<TableRow> = collect_ballots(voting_id)
        .await
        .rows
        .iter()
        .filter(|b| compare_styles_file_names(&b.voter, voter))
        .map(|b| b.clone())
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
    let voting = Voting::fill(voting_id, false).await;
    let ballots: Vec<TableRow> = collect_ballots(voting_id)
        .await
        .rows
        .iter()
        .filter(|b| compare_styles_file_names(&b.candidate, candidate))
        .map(|b| b.clone())
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
