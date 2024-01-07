use crate::{persistence::ToPersistence, serialize::FromStorage};
use chrono::{prelude::*, DateTime};
#[cfg(feature = "diesel_sqlite")]
use diesel::prelude::*;
use rocket::{
    debug, error,
    serde::{json::Json, Deserialize, Serialize},
};

pub const DELIMITER: &'static str = ",-o-,";
pub fn cleanup_delimiter(string: &str) -> String {
    string.replace(DELIMITER, ",")
}
pub fn add_delimiter(string: &str) -> String {
    string.to_owned() + DELIMITER
}
pub fn accumulate_strings_with_delimiter(string: &str, accumu: &str) -> String {
    string.to_owned() + ",-o-," + accumu
}
/////////////////////////////////////////////
//                                         //
//    TRAITS - TO BE IMPLEMENTED           //
//                                         //
/////////////////////////////////////////////

pub trait ToJson {
    fn to_empty_json() -> Json<Self>
    where
        Self: Sized;
}

pub trait Empty {
    fn empty() -> Self;
}

trait Filter {
    fn filter(self, criterion: &str) -> Self
    where
        Self: Sized;
}

pub trait IdGenerator {
    fn get_id(&self) -> String;
    fn generate_id(&self) -> String;
}

pub trait Selfaware {
    fn get_type(&self) -> String;
    fn get_name(&self) -> String;
}

pub trait Table {
    fn get_identity_column_name() -> String;
    fn get_table(insert: bool) -> String;
    fn get_db_columns() -> String;
    fn to_db_row(&self) -> String;
}
pub trait QueryableExt: IdGenerator + Table + Empty + Selfaware {}
#[rocket::async_trait]
pub trait Fill: Empty + FromStorage {
    fn convert(name: &str, matched_type: &str) -> String {
        match matched_type {
            "criteria" => name.to_string(),
            "candidate" => "candidate_".to_owned() + name,
            _ => name.to_string().to_lowercase(),
        }
    }
    async fn fill(name: &str, internal: bool, matched_type: &str) -> Self
    where
        Self: Sync + Sized + for<'de> Deserialize<'de>,
    {
        Self::empty()
            .load_into(&Self::convert(name, matched_type), internal)
            .await
    }
    async fn fill_json(name: &str, internal: bool, matched_type: &str) -> Json<Self>
    where
        Self: Sync + Sized + for<'de> Deserialize<'de>,
    {
        Json(Self::fill(name, internal, matched_type).await)
    }
}

/////////////////////////////////////////////
//                                         //
//             ID - ERROR                  //
//                                         //
/////////////////////////////////////////////

impl From<&str> for Voting {
    fn from(voting_id: &str) -> Self {
        Self::confident_voting(voting_id)
    }
}

trait ValidFileName {
    fn valid_file_name(self) -> bool;
}

pub fn from_optional_str(s: Option<&str>) -> String {
    match s {
        Some(b) => b.to_string(),
        None => String::new(),
    }
}
/////////////////////////////////////////////
//                                         //
//           JSON WRAPPER                  //
//                                         //
/////////////////////////////////////////////

pub enum VoteKind {
    Candidates(Candidates),
    Candidate(Candidate),
    Votings(Votings),
    Voting(Voting),
    Vote(Vote),
    Ballot(Ballot),
    KBallots(KnownBallots),
    CBallots(CastBallots),
    Criteria(Criteria),
    Criterion(Criterion),
}

impl From<Candidates> for VoteKind {
    fn from(candidates: Candidates) -> Self {
        VoteKind::Candidates(candidates)
    }
}

impl From<Candidate> for VoteKind {
    fn from(candidate: Candidate) -> Self {
        VoteKind::Candidate(candidate)
    }
}

impl From<Votings> for VoteKind {
    fn from(votings: Votings) -> Self {
        VoteKind::Votings(votings)
    }
}

impl From<Voting> for VoteKind {
    fn from(voting: Voting) -> Self {
        VoteKind::Voting(voting)
    }
}

impl From<Vote> for VoteKind {
    fn from(vote: Vote) -> Self {
        VoteKind::Vote(vote)
    }
}

impl From<KnownBallots> for VoteKind {
    fn from(known_ballots: KnownBallots) -> Self {
        VoteKind::KBallots(known_ballots)
    }
}
impl From<CastBallots> for VoteKind {
    fn from(cast_ballots: CastBallots) -> Self {
        VoteKind::CBallots(cast_ballots)
    }
}
impl From<Criteria> for VoteKind {
    fn from(criteria: Criteria) -> Self {
        VoteKind::Criteria(criteria)
    }
}
impl From<Criterion> for VoteKind {
    fn from(criterion: Criterion) -> Self {
        VoteKind::Criterion(criterion)
    }
}

/////////////////////////////////////////////
//                                         //
//                VOTINGS                  //
//                                         //
/////////////////////////////////////////////

#[derive(Debug, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Votings {
    pub votings: Vec<Voting>,
}
#[cfg(feature = "sqlx_sqlite")]
impl Votings {
    fn values(&self) -> String {
        self.votings
            .to_vec()
            .iter()
            .map(|v| v.values())
            .reduce(|acc, e| acc + &e)
            .unwrap()
    }
}
#[cfg(feature = "sqlx_sqlite")]
impl Table for Votings {
    fn get_identity_column_name() -> String {
        Voting::get_identity_column_name()
    }
    fn get_table(insert: bool) -> String {
        Voting::get_table(insert)
    }
    fn get_db_columns() -> String {
        Voting::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}
impl std::str::FromStr for Votings {
    type Err = crate::error::FromErrorKind;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        debug!("{:?}", v);
        let _result: Vec<&str> = v.split(",-o-,").collect();
        Ok(Self {
            votings: vec![], //TODO
        })
    }
}
impl ToJson for Votings {
    fn to_empty_json() -> Json<Votings> {
        Json(Votings::empty())
    }
}
impl Empty for Votings {
    fn empty() -> Votings {
        Votings { votings: vec![] }
    }
}
impl IdGenerator for Votings {
    fn get_id(&self) -> String {
        self.generate_id()
    }
    fn generate_id(&self) -> String {
        self.get_type()
    }
}
impl Selfaware for Votings {
    fn get_type(&self) -> String {
        String::from("Votings")
    }

    fn get_name(&self) -> String {
        String::from("Votings")
    }
}

impl Selfaware for Voting {
    fn get_type(&self) -> String {
        String::from("voting")
    }

    fn get_name(&self) -> String {
        self.name.to_lowercase().to_owned()
    }
}
impl IdGenerator for Voting {
    fn get_id(&self) -> String {
        self.generate_id()
    }
    fn generate_id(&self) -> String {
        self.name.to_lowercase().to_string()
    }
}

/////////////////////////////////////////////
//                                         //
//                 VOTING                  //
//                                         //
/////////////////////////////////////////////
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct VotingStyles {
    pub background: String,
    pub font: String,
    pub selection: String,
    pub fields: String,
}

impl VotingStyles {
    pub fn default() -> Self {
        Self {
            background: String::from("#30363d"),
            font: String::from("#ff0000"),
            selection: String::from("white"),
            fields: String::from("#238636"),
        }
    }
    pub fn values(&self) -> String {
        format!(
            "{}, {}, {}, {}",
            self.background, self.font, self.selection, self.fields
        )
    }
}
#[derive(Debug, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateVoting {
    pub name: String,
    pub expires_at: DateTime<Utc>,
    pub candidates: Vec<String>,
    pub criterias: Vec<String>,
    pub styles: Option<VotingStyles>,
    pub invite_code: String,
}
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Voting {
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub candidates: Vec<Candidate>,
    pub categories: Vec<Criterion>,
    pub styles: VotingStyles,
    pub invite_code: String,
}
#[cfg(feature = "sqlx_sqlite")]
impl Voting {
    fn properties(in_parenthesis: bool) -> String {
        if in_parenthesis {
            String::from(
                "( name, expires_at, created_at, candidates, categories, styles, invite_code )",
            )
        } else {
            String::from(
                "name, expires_at, created_at, candidates, categories, styles, invite_code",
            )
        }
    }
    fn values(&self) -> String {
        debug!("Voting values");
        VotingTable::from(self).values()
    }
}

#[cfg(feature = "sqlx_sqlite")]
impl Table for Voting {
    fn get_identity_column_name() -> String {
        String::from("name")
    }
    fn get_table(_insert: bool) -> String {
        String::from("votings")
    }
    fn get_db_columns() -> String {
        Voting::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}
#[cfg(feature = "db")]
impl From<Vec<VotingTable>> for Votings {
    fn from(rows: Vec<VotingTable>) -> Self {
        Self {
            votings: rows.iter().map(|v| Voting::from(v)).collect(),
        }
    }
}

// #[cfg(not(feature = "diesel_sqlite"))]
// #[derive(Debug, Serialize, PartialEq, Deserialize)]
// #[serde(crate = "rocket::serde")]
// pub struct VotingTable {
//     pub name: String,
//     pub expires_at: DateTime<Utc>,
//     pub created_at: DateTime<Utc>,
//     pub candidates: String,
//     pub categories: String,
//     pub styles: String,
//     pub invite_code: String,
// }
#[cfg(feature = "sqlx_sqlite")]
impl VotingTable {
    fn get_name(&self) -> String {
        self.name.to_lowercase().to_owned()
    }
    fn properties(in_parenthesis: bool) -> String {
        Voting::properties(in_parenthesis)
    }
    fn values(&self) -> String {
        debug!("VotingTable values");
        vec![
            "\"".to_owned() + &self.name.clone() + "\"",
            self.expires_at.to_string(),
            self.created_at.to_string(),
            self.candidates.clone(),
            self.categories.clone(),
            self.styles.clone(),
            self.invite_code.clone(),
        ]
        .into_iter()
        .reduce(|acc, e| acc + ", '" + &e + "'")
        .unwrap()
    }
}
#[cfg(feature = "sqlx_sqlite")]
#[derive(Debug, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct VotingTable {
    pub name: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub candidates: String,
    pub categories: String,
    pub styles: String,
    pub invite_code: String,
}

#[cfg(feature = "diesel_sqlite")]
#[derive(Debug, Serialize, PartialEq, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = votings )]
#[serde(crate = "rocket::serde")]
pub struct VotingTable {
    pub name: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub candidates: String,
    pub categories: String,
    pub styles: String,
    pub invite_code: String,
}
#[cfg(feature = "sqlx_sqlite")]
impl Table for VotingTable {
    fn get_identity_column_name() -> String {
        String::from("name")
    }
    fn get_table(insert: bool) -> String {
        Voting::get_table(insert)
    }
    fn get_db_columns() -> String {
        Voting::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}

impl std::str::FromStr for Voting {
    type Err = crate::error::FromErrorKind;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        debug!("{:?}", v);
        let result: Vec<&str> = v.split(",-o-,").collect();
        if result.len() >= 7 {
            debug!("We got all 7 rows back!:");
            Ok(Self {
                name: from_optional_str(result.get(0).copied()),
                expires_at: Some(DateTime::from_str(result.get(1).copied().unwrap()).unwrap()),
                created_at: Some(DateTime::from_str(result.get(2).copied().unwrap()).unwrap()),
                candidates: match rocket::serde::json::from_str::<Vec<Candidate>>(
                    &from_optional_str(result.get(3).copied()),
                ) {
                    Ok(stringified) => stringified,
                    Err(e) => {
                        error!("{:?}", e);
                        vec![]
                    }
                },
                categories: match rocket::serde::json::from_str::<Vec<Criterion>>(
                    &from_optional_str(result.get(4).copied()),
                ) {
                    Ok(stringified) => stringified,
                    Err(e) => {
                        error!("{:?}", e);
                        vec![]
                    }
                },
                styles: match rocket::serde::json::from_str::<VotingStyles>(&from_optional_str(
                    result.get(5).copied(),
                )) {
                    Ok(stringified) => stringified,
                    Err(e) => {
                        error!("{:?}", e);
                        VotingStyles::default()
                    }
                },
                invite_code: from_optional_str(result.get(6).copied()),
            })
        } else {
            debug!("Nope, sth didnt wor.");
            Err(crate::error::FromErrorKind::Serialize(String::from(
                "Voting serialize failed.",
            )))
        }
    }
}
#[cfg(feature = "sqlx_sqlite")]
impl From<&VotingTable> for Voting {
    fn from(v: &VotingTable) -> Self {
        Self {
            name: String::from(&v.name),
            expires_at: Some(v.expires_at),
            created_at: Some(v.created_at),
            candidates: match rocket::serde::json::from_str::<Vec<Candidate>>(&v.candidates) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    vec![]
                }
            },
            categories: match rocket::serde::json::from_str::<Vec<Criterion>>(&v.categories) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    vec![]
                }
            },
            styles: match rocket::serde::json::from_str::<VotingStyles>(&v.styles) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    VotingStyles::default()
                }
            },
            invite_code: String::from(&v.invite_code),
        }
    }
}
#[cfg(feature = "sqlx_sqlite")]
impl From<VotingTable> for Voting {
    fn from(v: VotingTable) -> Self {
        Self {
            name: v.name,
            expires_at: Some(v.expires_at),
            created_at: Some(v.created_at),
            candidates: match rocket::serde::json::from_str::<Vec<Candidate>>(&v.candidates) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    vec![]
                }
            },
            categories: match rocket::serde::json::from_str::<Vec<Criterion>>(&v.categories) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    vec![]
                }
            },
            styles: match rocket::serde::json::from_str::<VotingStyles>(&v.styles) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    VotingStyles::default()
                }
            },
            invite_code: v.invite_code,
        }
    }
}
#[cfg(feature = "diesel_sqlite")]
table! {
    votings (name) {
        name -> Text,
        expires_at -> TimestamptzSqlite,
        created_at -> TimestamptzSqlite,
        candidates -> Text,
        categories -> Text,
        styles -> Text,
        invite_code -> Text,
    }
}
#[cfg(feature = "sqlx_sqlite")]
impl From<&Voting> for VotingTable {
    fn from(v: &Voting) -> Self {
        Self {
            name: v.name.clone(),
            expires_at: v.expires_at.unwrap(),
            created_at: v.created_at.unwrap(),
            candidates: match rocket::serde::json::to_string(&v.candidates) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    String::new()
                }
            },
            categories: match rocket::serde::json::to_string(&v.categories) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    String::new()
                }
            },
            styles: match rocket::serde::json::to_string(&v.styles) {
                Ok(stringified) => stringified,
                Err(e) => {
                    error!("{:?}", e);
                    String::new()
                }
            },
            invite_code: v.invite_code.clone(),
        }
    }
}
impl Voting {
    fn confident_voting(id: &str) -> Self {
        Self {
            created_at: Some(Utc::now()),
            expires_at: Some(Local::now().into()),
            name: id.to_string(),
            candidates: vec![],
            categories: vec![],
            styles: VotingStyles::default(),
            invite_code: String::from("access"),
        }
    }
}

impl ToJson for Voting {
    fn to_empty_json() -> Json<Voting> {
        Json(Voting::empty())
    }
}

impl Empty for Voting {
    fn empty() -> Voting {
        Voting {
            name: String::from("Empty"),
            created_at: Some(Utc::now()),
            expires_at: Some(Local::now().into()),
            candidates: Candidates::empty().candidates,
            categories: vec![],
            styles: VotingStyles::default(),
            invite_code: String::from("access"),
        }
    }
}

impl Fill for Voting {}

/////////////////////////////////////////////
//                                         //
//              CANDIDATE                  //
//                                         //
/////////////////////////////////////////////

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NonVoter<'r>(&'r str);

#[cfg(not(feature = "diesel_sqlite"))]
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Candidate {
    pub voter: bool,
    pub id: Option<String>,
    pub label: String,
}

impl Candidate {
    fn properties(in_parenthesis: bool) -> String {
        if in_parenthesis {
            String::from("( voter, id, label )")
        } else {
            String::from("voter, id, label")
        }
    }
    fn values(&self) -> String {
        vec![
            match self.voter {
                true => String::from("true"),
                false => String::from("false"),
            },
            "\"".to_owned() + &self.get_id() + "\"",
            "\"".to_owned() + &self.label.clone() + "\"",
        ]
        .into_iter()
        .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
        .unwrap()
    }
}
impl std::str::FromStr for Candidate {
    type Err = crate::error::FromErrorKind;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        debug!("{:?}", v);
        let result: Vec<&str> = v.split(",-o-,").collect();
        Ok(Self {
            voter: true, // TODO from_optional_str(result.get(0).copied()),
            id: Some(from_optional_str(result.get(1).copied())),
            label: from_optional_str(result.get(2).copied()),
        })
    }
}
#[cfg(feature = "diesel_sqlite")]
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, Queryable, Insertable)]
#[diesel(table_name = candidates)]
#[serde(crate = "rocket::serde")]
pub struct Candidate {
    pub voter: bool,
    pub id: Option<String>,
    pub label: String,
}

#[cfg(feature = "sqlx_sqlite")]
impl Table for Candidate {
    fn get_identity_column_name() -> String {
        String::from("id")
    }
    fn get_table(insert: bool) -> String {
        if insert {
            String::from("candidates ") + &Candidate::properties(true)
        } else {
            String::from("candidates")
        }
    }
    fn get_db_columns() -> String {
        Candidate::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}
#[cfg(feature = "diesel_sqlite")]
table! {
    candidates (id) {
        id -> Nullable<Text>,
        label -> Text,
        voter -> Bool,
    }
}
impl Candidate {
    fn get_name(&self) -> String {
        match &self.id {
            Some(i) => i.to_string(),
            None => String::from("n/a"),
        }
    }
    pub fn get_label_from_id(id: &str) -> String {
        let inserted_spaces = id.replace("-#s-", " ");
        let inserted_extra_ident = inserted_spaces.as_str().replace("%-%", "-#s-");
        inserted_extra_ident
    }
    fn get_id_from_label(label: &str) -> String {
        let replaced_underscores = label.replace("-#s-", "%-%");
        let replaced_spaces = replaced_underscores.as_str().replace(' ', "-#s-");
        replaced_spaces
    }
    pub fn set_id(&mut self, id: &str) {
        self.id = Some(id.to_string());
    }
    pub fn voter(label: &str) -> Candidate {
        Candidate {
            voter: true,
            id: None,
            label: label.to_string(),
        }
    }
}

impl ToJson for Candidate {
    fn to_empty_json() -> Json<Candidate> {
        Json(Candidate::empty())
    }
}

impl Empty for Candidate {
    fn empty() -> Candidate {
        Candidate {
            voter: false,
            id: Some(String::new()),
            label: String::new(),
        }
    }
}

impl IdGenerator for Candidate {
    fn get_id(&self) -> String {
        match &self.id {
            Some(i) => i.to_string(),
            None => self.generate_id(),
        }
    }
    fn generate_id(&self) -> String {
        match self.voter {
            false => {
                "candidate_".to_owned() + &Candidate::get_id_from_label(&self.label.to_lowercase())
            }
            true => "voter_".to_owned() + &Candidate::get_id_from_label(&self.label.to_lowercase()),
        }
    }
}

impl Selfaware for Candidate {
    fn get_type(&self) -> String {
        match self.voter {
            false => String::from("Candidate"),
            true => String::from("Voter"),
        }
    }

    fn get_name(&self) -> String {
        match &self.id {
            Some(k) => k.to_lowercase().to_owned(),
            None => String::from(self.label.as_str()),
        }
    }
}
#[rocket::async_trait]
impl Fill for Candidate {
    async fn fill(name: &str, internal: bool, _matched_type: &str) -> Self
    where
        Self: Sync + Sized + for<'de> Deserialize<'de>,
    {
        Self::empty()
            .load_into(
                &("candidate_".to_string() + &(name.to_lowercase())),
                internal,
            )
            .await
    }
}

/////////////////////////////////////////////
//                                         //
//             CANDIDATES                  //
//                                         //
/////////////////////////////////////////////

#[derive(Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Candidates {
    pub candidates: Vec<Candidate>,
}

impl Candidates {
    pub fn voters(self) -> Candidates {
        Candidates {
            candidates: self
                .candidates
                .iter()
                .filter(|c| c.voter)
                .map(|c| Candidate {
                    voter: c.voter,
                    id: c.id.to_owned(),
                    label: c.label.to_owned(),
                })
                .collect(),
        }
    }
    pub fn candidates(self) -> Candidates {
        Candidates {
            candidates: self
                .candidates
                .iter()
                .filter(|c| !c.voter)
                .map(|c| Candidate {
                    voter: c.voter,
                    id: c.id.to_owned(),
                    label: c.label.to_owned(),
                })
                .collect(),
        }
    }
}
impl ToJson for Candidates {
    fn to_empty_json() -> Json<Candidates> {
        Json(Candidates::empty())
    }
}

impl Empty for Candidates {
    fn empty() -> Candidates {
        Candidates { candidates: vec![] }
    }
}

/////////////////////////////////////////////
//                                         //
//            CAST BALLOT                  //
//                                         //
////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CastBallots {
    pub voting: Option<String>,
    pub ballots: Vec<KnownBallots>,
}
#[cfg(feature = "diesel_sqlite")]
table! {
    ballots (voter) {
        voter -> Text,
        human_identifier -> Text,
        candidate -> Text,
        sum -> SmallInt,
        weighted -> Float,
        mean -> Float,
        notes -> Text,
        votes -> Text,
        voted_on -> TimestamptzSqlite,
    }
}
impl CastBallots {
    pub fn properties(in_parenthesis: bool) -> String {
        let raw = "human_identifier, candidate, voter, sum, weighted, mean, notes, votes, voted_on";
        if in_parenthesis {
            "( ".to_owned() + raw + " )"
        } else {
            raw.to_string()
        }
    }
}

#[cfg(feature = "sqlx_sqlite")]
impl Table for CastBallots {
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
        error!("NOT IMPLEMENTED");
        String::new()
    }
}
impl std::str::FromStr for CastBallots {
    type Err = crate::error::FromErrorKind;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        debug!("{:?}", v);
        let result: Vec<&str> = v.split(",-o-,").collect();
        Ok(Self {
            voting: Some(from_optional_str(result.get(0).copied())),
            ballots: vec![], // TODO result.get(0).unwrap()
        })
    }
}
impl ToJson for CastBallots {
    fn to_empty_json() -> Json<Self> {
        Json(Self::empty())
    }
}

impl Empty for CastBallots {
    fn empty() -> Self {
        Self {
            voting: Some(String::new()),
            ballots: vec![],
        }
    }
}

impl IdGenerator for CastBallots {
    fn get_id(&self) -> String {
        self.generate_id()
    }
    fn generate_id(&self) -> String {
        let voting_name = match &self.voting {
            Some(v) => String::from(v.as_str()),
            None => String::from("dummy-voting"),
        };
        match self.ballots.first() {
            Some(f) => match f.ballots.first() {
                Some(b) => {
                    voting_name.to_lowercase()
                        + "-"
                        + &f.voter.to_lowercase()
                        + "-"
                        + &b.candidate.to_lowercase()
                }
                None => voting_name.to_lowercase() + "-" + &f.voter.to_lowercase(),
            },
            None => voting_name + "-index",
        }
    }
}

impl Selfaware for CastBallots {
    fn get_type(&self) -> String {
        String::from("CBallots")
    }

    fn get_name(&self) -> String {
        String::from("CBallots")
    }
}

impl Fill for CastBallots {}

/////////////////////////////////////////////
//                                         //
//           KNOWN BALLOT                  //
//                                         //
/////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct KnownBallots {
    pub voter: String,
    pub ballots: Vec<Ballot>,
}
impl KnownBallots {
    fn get_name(&self) -> String {
        self.voter.to_lowercase().to_owned()
    }
    fn values(&self) -> String {
        vec![
            self.voter.clone(),
            self.ballots
                .iter()
                .map(|b| b.values())
                .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
                .unwrap(),
        ]
        .into_iter()
        .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
        .unwrap()
    }
}

impl ToJson for KnownBallots {
    fn to_empty_json() -> Json<Self> {
        Json(Self::empty())
    }
}

impl Empty for KnownBallots {
    fn empty() -> KnownBallots {
        KnownBallots {
            voter: String::new(),
            ballots: vec![],
        }
    }
}

/////////////////////////////////////////////
//                                         //
//                 BALLOT                  //
//                                         //
/////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Ballot {
    pub candidate: String,
    pub votes: Vec<Vote>,
    pub notes: Option<String>,
    pub voted_on: Option<DateTime<Utc>>,
}

impl Ballot {
    fn values(&self) -> String {
        vec![
            self.candidate.clone(),
            self.votes
                .iter()
                .map(|v| v.values())
                .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
                .unwrap(),
            match &self.notes {
                Some(n) => n.to_string(),
                None => String::from("no-notes"),
            },
            match &self.voted_on {
                Some(v) => v.to_string(),
                None => String::from("n/a"),
            },
        ]
        .into_iter()
        .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
        .unwrap()
    }
}
/////////////////////////////////////////////
//                                         //
//                 VOTE                    //
//                                         //
/////////////////////////////////////////////

#[derive(Serialize, Clone, Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Vote {
    pub name: String,
    pub point: i8,
}
impl Vote {
    fn get_name(&self) -> String {
        self.name.to_lowercase().to_owned()
    }
    fn properties(in_parenthesis: bool) -> String {
        if in_parenthesis {
            String::from("( name, point )")
        } else {
            String::from("name, point")
        }
    }
    fn values(&self) -> String {
        vec![self.name.clone(), self.point.to_string()]
            .into_iter()
            .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
            .unwrap()
    }
}
#[cfg(not(feature = "diesel_sqlite"))]
#[derive(Serialize, Clone, Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct VoteTable {
    pub name: String,
    pub point: i16,
}
impl VoteTable {
    fn get_name(&self) -> String {
        self.name.to_lowercase().to_owned()
    }
    fn properties(in_parenthesis: bool) -> String {
        Vote::properties(in_parenthesis)
    }
    fn values(&self) -> String {
        vec![
            "\"".to_owned() + &self.name.clone() + "\"",
            self.point.to_string(),
        ]
        .into_iter()
        .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
        .unwrap()
    }
}
#[cfg(feature = "diesel_sqlite")]
#[derive(Serialize, Clone, Debug, PartialEq, Deserialize, Queryable, Insertable)]
#[diesel(table_name = votes)]
#[serde(crate = "rocket::serde")]
pub struct VoteTable {
    pub name: String,
    pub point: i16,
}
#[cfg(feature = "sqlx_sqlite")]
impl Table for VoteTable {
    fn get_identity_column_name() -> String {
        VoteTable::get_identity_column_name()
    }
    fn get_table(insert: bool) -> String {
        VoteTable::get_table(insert)
    }
    fn get_db_columns() -> String {
        VoteTable::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}
#[cfg(feature = "diesel_sqlite")]
table! {
    votes (name) {
        name -> Text,
        point -> SmallInt
    }
}
impl From<Vote> for VoteTable {
    fn from(v: Vote) -> Self {
        Self {
            name: v.name,
            point: i16::from(v.point),
        }
    }
}
impl ToJson for Vote {
    fn to_empty_json() -> Json<Self> {
        Json(Self::empty())
    }
}

impl Empty for Vote {
    fn empty() -> Self {
        Self {
            name: String::new(),
            point: 0,
        }
    }
}

/////////////////////////////////////////////
//                                         //
//              CRITERION                  //
//                                         //
/////////////////////////////////////////////

#[derive(Serialize, Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Criteria {
    pub criterias: Vec<Criterion>,
}
impl Criteria {
    fn properties(in_parenthesis: bool) -> String {
        Criterion::properties(in_parenthesis)
    }
    #[cfg(feature = "sqlx_sqlite")]
    fn values(&self) -> String {
        self.criterias
            .iter()
            .map(|c| c.to_db_row())
            .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
            .unwrap()
    }
}

#[cfg(feature = "sqlx_sqlite")]
impl Table for Criteria {
    fn get_identity_column_name() -> String {
        String::from("name")
    }
    fn get_table(insert: bool) -> String {
        Criterion::get_table(insert)
    }
    fn get_db_columns() -> String {
        Criterion::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}
impl std::str::FromStr for Criteria {
    type Err = crate::error::FromErrorKind;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        debug!("{:?}", v);
        let _result: Vec<&str> = v.split(",-o-,").collect();
        Ok(Self {
            criterias: vec![], // TODO result.get(0).unwrap()
        })
    }
}
impl ToJson for Criteria {
    fn to_empty_json() -> Json<Self> {
        Json(Self::empty())
    }
}

impl Empty for Criteria {
    fn empty() -> Self {
        Self { criterias: vec![] }
    }
}
impl IdGenerator for Criteria {
    fn get_id(&self) -> String {
        self.generate_id()
    }
    fn generate_id(&self) -> String {
        self.get_type()
    }
}
impl Selfaware for Criteria {
    fn get_type(&self) -> String {
        String::from("Criteria")
    }

    fn get_name(&self) -> String {
        String::from("Criteria")
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Criterion {
    pub name: String,
    pub min: i8,
    pub max: i8,
    pub weight: Option<f32>,
}
impl Criterion {
    fn properties(in_parenthesis: bool) -> String {
        if in_parenthesis {
            String::from("( name, min, max, weight )")
        } else {
            String::from("name, min, max, weight")
        }
    }
    fn values(&self) -> String {
        vec![
            "\"".to_owned() + &self.name.clone() + "\"",
            self.min.to_string(),
            self.max.to_string(),
            match self.weight {
                Some(w) => w.to_string(),
                None => String::from("1.0"),
            },
        ]
        .into_iter()
        .reduce(|acc, e| acc + ", " + &e)
        .unwrap()
    }
}
#[cfg(feature = "sqlx_sqlite")]
impl Table for Criterion {
    fn get_identity_column_name() -> String {
        String::from("name")
    }
    fn get_table(insert: bool) -> String {
        if insert {
            String::from("criteria ") + &Criterion::properties(true)
        } else {
            String::from("criteria ")
        }
    }
    fn get_db_columns() -> String {
        Criterion::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}

impl std::str::FromStr for Criterion {
    type Err = crate::error::FromErrorKind;

    fn from_str(v: &str) -> Result<Self, Self::Err> {
        debug!("Raw criterion str: {:?}", v);
        let result: Vec<&str> = v.split(DELIMITER).collect();
        debug!("Split: {:?}", result);
        Ok(Self {
            name: from_optional_str(Some(result.get(0).expect("This index exist.").trim())),
            min: i8::from_str(&from_optional_str(Some(
                result.get(1).expect("This index exist.").trim(),
            )))
            .unwrap(),
            max: i8::from_str(&from_optional_str(Some(
                result.get(2).expect("This index exist.").trim(),
            )))
            .unwrap(),
            weight: Some(
                f32::from_str(&from_optional_str(Some(
                    result.get(3).expect("This index exist.").trim(),
                )))
                .unwrap(),
            ),
        })
    }
}

impl From<&CriterionRow> for Criterion {
    fn from(c_row: &CriterionRow) -> Self {
        Self {
            name: c_row.name.clone(),
            min: i8::try_from(c_row.min).unwrap(),
            max: i8::try_from(c_row.max).unwrap(),
            weight: c_row.weight,
        }
    }
}
#[cfg(not(feature = "diesel_sqlite"))]
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CriterionRow {
    pub name: String,
    pub min: i16,
    pub max: i16,
    pub weight: Option<f32>,
}
impl CriterionRow {
    fn values(&self) -> String {
        Criterion::from(self).values()
    }
}
#[cfg(feature = "diesel_sqlite")]
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, Queryable, Insertable)]
#[diesel(table_name = criteria)]
#[serde(crate = "rocket::serde")]
pub struct CriterionRow {
    pub name: String,
    pub min: i16,
    pub max: i16,
    pub weight: Option<f32>,
}
#[cfg(feature = "sqlx_sqlite")]
impl Table for CriterionRow {
    fn get_identity_column_name() -> String {
        Criterion::get_identity_column_name()
    }
    fn get_table(insert: bool) -> String {
        Criterion::get_table(insert)
    }
    fn get_db_columns() -> String {
        Criterion::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}
#[cfg(feature = "diesel_sqlite")]
table! {
    criteria (name) {
        name -> Text,
        min -> SmallInt,
        max -> SmallInt,
        weight -> Nullable<Float>,
    }
}

impl From<Criterion> for CriterionRow {
    fn from(cr: Criterion) -> Self {
        Self {
            name: cr.name,
            min: i16::from(cr.min),
            max: i16::from(cr.max),
            weight: cr.weight,
        }
    }
}
impl Fill for Criterion {}
impl Selfaware for Criterion {
    fn get_type(&self) -> String {
        String::from("Criterion")
    }

    fn get_name(&self) -> String {
        debug!("Criterion::get_name with trait Selfaware");
        self.name.to_lowercase().to_owned()
    }
}

impl IdGenerator for Criterion {
    fn get_id(&self) -> String {
        self.generate_id()
    }
    fn generate_id(&self) -> String {
        let weight = match self.weight {
            Some(w) => w.to_string(),
            None => 1.0.to_string(),
        };
        [
            self.name.to_lowercase().to_owned(),
            self.min.to_string(),
            self.max.to_string(),
            weight,
        ]
        .into_iter()
        .reduce(|acc, e| acc + "_" + &e)
        .unwrap()
    }
}

impl ToJson for Criterion {
    fn to_empty_json() -> Json<Self> {
        Json(Self::empty())
    }
}

impl Empty for Criterion {
    fn empty() -> Self {
        Self {
            name: String::new(),
            min: 0,
            max: 10,
            weight: None,
        }
    }
}
#[derive(Debug, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Users {
    pub voters: Vec<String>,
    pub candidates: Vec<String>,
}

pub async fn get_users_internal() -> Users {
    let mut voters = vec![];
    let mut candidates = vec![];
    if let Ok(r) = Candidate::empty().index().await {
        r.into_iter().for_each(|c| {
            match c.starts_with("voter") {
                true => voters.push(c.strip_prefix("voter_").unwrap().to_string()),
                false => candidates.push(c.strip_prefix("candidate_").unwrap().to_string()),
            };
        });
    }
    Users { voters, candidates }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct EmojiCategories {
    pub emojis: Vec<EmojiCategory>,
}
impl EmojiCategories {
    fn values(&self) -> String {
        self.emojis
            .to_vec()
            .iter()
            .map(|e| e.values())
            .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
            .unwrap()
    }
}
#[cfg(feature = "sqlx_sqlite")]
impl Table for EmojiCategories {
    fn get_identity_column_name() -> String {
        EmojiCategory::get_identity_column_name()
    }
    fn get_table(insert: bool) -> String {
        if insert {
            String::from("emojis") + &EmojiCategory::properties(true)
        } else {
            String::from("emojis")
        }
    }
    fn get_db_columns() -> String {
        EmojiCategory::properties(false)
    }
    fn to_db_row(&self) -> String {
        self.values()
    }
}
impl std::str::FromStr for EmojiCategories {
    type Err = crate::error::FromErrorKind;
    fn from_str(v: &str) -> Result<Self, Self::Err> {
        debug!("{:?}", v);
        let _result: Vec<&str> = v.split(",-o-,").collect();
        Ok(Self {
            emojis: vec![], // TODO result.get(0).unwrap()
        })
    }
}
impl Empty for EmojiCategories {
    fn empty() -> Self {
        Self { emojis: vec![] }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct EmojiCategory {
    pub name: String,
    emojis: Vec<String>,
}

impl EmojiCategory {
    pub fn new(name: &str, emojis: &Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            emojis: emojis.to_vec(),
        }
    }

    fn get_identity_column_name() -> String {
        String::from("name")
    }
    fn properties(in_parenthesis: bool) -> String {
        if in_parenthesis {
            String::from("( name, emojis )")
        } else {
            String::from("name, emojis")
        }
    }
    fn values(&self) -> String {
        vec![
            self.name.clone(),
            self.emojis
                .clone()
                .into_iter()
                .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
                .unwrap(),
        ]
        .into_iter()
        .reduce(|acc, e| accumulate_strings_with_delimiter(&acc, &e))
        .unwrap()
    }
}
impl IdGenerator for EmojiCategories {
    fn get_id(&self) -> String {
        self.generate_id()
    }
    fn generate_id(&self) -> String {
        String::from("emojis")
    }
}
impl Selfaware for EmojiCategories {
    fn get_type(&self) -> String {
        String::from("emojis")
    }

    fn get_name(&self) -> String {
        String::from("emojis")
    }
}
