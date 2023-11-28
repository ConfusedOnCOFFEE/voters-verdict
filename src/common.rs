use crate::config::{ASSET_DIR, FILE_DIR};
use chrono::{prelude::*, DateTime};
use log::{debug, error};
use rocket::{
    response::status::Unauthorized,
    serde::{json::Json, Deserialize, Serialize},
    tokio::{
        fs::File,
        io::{AsyncReadExt, AsyncWriteExt},
    },
    Responder,
};
use std::io::Read;

enum PersistenceMode<'a> {
    DB(DatabaseConfig<'a>),
    File(FileConfig),
    Remote(RemoteConfig<'a>),
}
struct RemoteConfig<'a> {
    url: Option<&'a str>,
}
struct FileConfig {
    dir: String,
}

struct DatabaseConfig<'a> {
    url: Option<&'a str>,
}

impl<'a> PersistenceMode<'a> {
    fn detect_config_mode() -> Self {
        match option_env!["VOTERS_VERDICT_STORAGE_MODE"] {
            Some(s) => PersistenceMode::from(s),
            None => PersistenceMode::File(FileConfig {
                dir: match std::env::var("VOTERS_VERDICT_FILE_DIR") {
                    Ok(file_dir) => file_dir,
                    Err(_) => String::from("/tmp/"),
                },
            }),
        }
    }
    fn to_conform_path() -> String {
        match PersistenceMode::detect_config_mode() {
            PersistenceMode::DB(d) => match d.url {
                Some(s) => s.to_string(),
                None => panic!("No DB"),
            },
            PersistenceMode::File(d) => d.dir,
            PersistenceMode::Remote(d) => {
                debug!("Remote URL: {:?}", d.url);
                match d.url {
                    Some(s) => s.to_string(),
                    None => panic!("No remote"),
                }
            }
        }
    }
}
impl<'a> From<&str> for PersistenceMode<'a> {
    fn from(candidates: &str) -> Self {
        match candidates {
            "1" | "db" => PersistenceMode::DB(DatabaseConfig {
                url: option_env!("VOTERS_VERDICT_DB_URL"),
            }),
            "2" | "file" => PersistenceMode::File(FileConfig {
                dir: match std::env::var(FILE_DIR) {
                    Ok(file_dir) => file_dir,
                    Err(_) => String::from("/tmp/"),
                },
            }),
            "3" | "remote" => PersistenceMode::Remote(RemoteConfig {
                url: option_env!["VOTERS_VERDICT_REMOTE_STORAGE"],
            }),
            _ => PersistenceMode::File(FileConfig {
                dir: match std::env::var("VOTERS_VERDICT_FILE_DIR") {
                    Ok(file_dir) => file_dir,
                    Err(_) => String::from("/tmp/"),
                },
            }),
        }
    }
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
}
pub trait FileDir {
    fn get_dir() -> &'static str;
    fn get_full_path(&self, possible_remote: bool) -> String {
        let root_path = match possible_remote {
            true => PersistenceMode::to_conform_path(),
            false => match std::env::var(FILE_DIR) {
                Ok(file_dir) => file_dir,
                Err(_) => String::from("/tmp/"),
            },
        };
        root_path + &Self::get_dir()
    }
}

/////////////////////////////////////////////
//                                         //
//    TRAIT - IMPLEMENTED                  //
//                                         //
/////////////////////////////////////////////
#[rocket::async_trait]
pub trait Index: FileDir + FromJsonFile {
    async fn index(&self) -> Result<Vec<String>, String> {
        let uri = self.get_full_path(true) + "/index.json";
        match PersistenceMode::detect_config_mode() {
            PersistenceMode::DB(_d) => {
                panic!("n/a. Please use DB or File as VOTERS_VERDICT_STORAGE_MODE")
            }
            PersistenceMode::File(_d) => self.read_dir(&uri).await,
            PersistenceMode::Remote(_d) => {
                debug!("Remote URL: {:?}", uri);
                match std::env::var("VOTERS_VERDICT_SELF_CERT") {
                    Ok(cert) => {
                        let mut buf = Vec::new();
                        match File::open(cert).await {
                            Ok(mut f) => {
                                let _ = match f.read_to_end(&mut buf).await {
                                    Ok(_) => {}
                                    Err(e) => error!("{:?}", e),
                                };

                                match reqwest::Certificate::from_der(&buf) {
                                    Ok(c) => {
                                        let client_builder =
                                            reqwest::Client::builder().add_root_certificate(c);
                                        self.build_client(&uri, Some(client_builder)).await
                                    }
                                    Err(_e) => {
                                        error!("Self signed cert couldn't be loaded.");
                                        self.build_client(&uri, None).await
                                    }
                                }
                            }
                            Err(_) => {
                                error!("Self signed cert couldn't be loaded.");
                                self.build_client(&uri, None).await
                            }
                        }
                    }
                    Err(_) => self.build_client(&uri, None).await,
                }
            }
        }
    }
    async fn build_client(
        &self,
        uri: &str,
        client_builder: Option<reqwest::ClientBuilder>,
    ) -> Result<Vec<String>, String> {
        let custom_client = match client_builder {
            Some(builder) => builder,
            None => reqwest::Client::builder(),
        };
        let client = match custom_client.build() {
            Ok(c) => c,
            Err(_) => reqwest::Client::new(),
        };
        self.http_get(uri, client).await
    }

    async fn http_get(&self, uri: &str, client: reqwest::Client) -> Result<Vec<String>, String> {
        let get_client = client.get(uri);
        match std::env::var("VOTERS_VERDICT_REMOTE_CREDENTIALS") {
            Ok(t) => match std::env::var("VOTERS_VERDICT_REMOTE_AUTH") {
                Ok(auths) => {
                    debug!("Detected remote auth: {}", auths);
                    match auths.as_str() {
                        "bearer" => {
                            debug!("Detected bearer auth");
                            self.get(get_client.bearer_auth("Bearer ".to_owned() + &t))
                                .await
                        }
                        "basic" => {
                            debug!("Detected basic auth");
                            let credentails = auths.split_once(':');
                            match credentails {
                                Some((user, pw)) => {
                                    self.get(get_client.basic_auth(user, Some(pw))).await
                                }
                                None => {
                                    error!("Credentials for basic auth not provided");
                                    self.get(get_client).await
                                }
                            }
                        }
                        _ => self.get(get_client).await,
                    }
                }
                Err(_) => self.get(get_client).await,
            },
            Err(_) => self.get(get_client).await,
        }
    }
    async fn get(&self, request_builder: reqwest::RequestBuilder) -> Result<Vec<String>, String> {
        match request_builder
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .send()
            .await
        {
            Ok(r) => {
                debug!("Raw request response: {:?}", r);
                match r.json::<Vec<String>>().await {
                    Ok(d) => Ok(d),
                    Err(e) => {
                        error!("{:?}", e);
                        Ok(vec![])
                    }
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Ok(vec![])
            }
        }
    }

    async fn list(&self) -> Result<Vec<String>, String> {
        let full_path = self.get_full_path(true);
        debug!("{:?}", full_path);
        let read_dir = tokio::fs::read_dir(full_path);
        match read_dir.await {
            Ok(mut entries) => {
                let mut files = vec![];
                while let entry = entries.next_entry() {
                    match entry.await {
                        Ok(e) => match e {
                            Some(unpacked_e) => match unpacked_e.file_name().into_string() {
                                Ok(file_name) => match file_name.strip_suffix(".json") {
                                    Some(cleaned) => {
                                        files.push(cleaned.to_owned());
                                    }
                                    None => {}
                                },
                                Err(_) => {}
                            },
                            None => break,
                        },
                        Err(_) => break,
                    }
                }
                Ok(files)
            }
            Err(_) => Ok(vec![]),
        }
    }
    async fn read_dir(&self, uri: &str) -> Result<Vec<String>, String> {
        debug!("Reading file from: {:?}", uri);
        let raw_file = self.from_file(uri).await;
        match rocket::serde::json::from_str::<Vec<String>>(&raw_file) {
            Ok(json) => Ok(json),
            Err(_votes) => Ok(vec![]),
        }
    }
}
#[rocket::async_trait]
pub trait IsUnique: Index {
    async fn is_unique(&self, id: &str) -> Result<String, VoteErrorKind>
    where
        Self: Sync,
    {
        match self.index().await {
            Ok(files_exist) => match files_exist.iter().find(|v| match v.split_once('_') {
                Some((_v_type, found_id)) => found_id.to_lowercase() == id.to_lowercase(),
                None => true,
            }) {
                Some(s) => Ok(s.to_string()),
                None => Err(VoteErrorKind::NotFound(String::from("Not found"))),
            },
            Err(_) => Err(VoteErrorKind::NotFound(String::from("Not found"))),
        }
    }
}

#[rocket::async_trait]
pub trait FromJsonFile: Empty + FileDir {
    async fn read_to_string(&self, f: File) -> String {
        let mut contents = String::new();
        f.into_std().await.read_to_string(&mut contents).unwrap();
        contents
    }
    async fn from_file(&self, path: &str) -> String {
        match File::open(path).await {
            Ok(r) => self.read_to_string(r).await,
            Err(e) => {
                error!("Error: {:?} on {:?}", e, path);
                String::new()
            }
        }
    }
    async fn from_json_file(&self, id: &str, internal: bool) -> Json<Self>
    where
        Self: Sized + for<'de> Deserialize<'de>,
    {
        Json(self.from_json_file_to_self(id, internal).await)
    }

    async fn from_json_file_to_self(&self, id: &str, internal: bool) -> Self
    where
        Self: Sized + for<'de> Deserialize<'de>,
    {
        let path = self.get_full_path(true) + "/" + id + ".json";
        let raw_file = match PersistenceMode::detect_config_mode() {
            PersistenceMode::DB(_d) => {
                panic!("n/a. Please use DB or File as VOTERS_VERDICT_STORAGE_MODE")
            }
            PersistenceMode::File(_d) => self.from_file(&path).await,
            PersistenceMode::Remote(_d) => {
                if internal {
                    self.from_file(&path).await
                } else {
                    debug!("Full path: {:?}", self.get_full_path(true));
                    match reqwest::get(self.get_full_path(true) + "/" + id + ".json").await {
                        Ok(r) => match r.text().await {
                            Ok(as_text) => as_text,
                            Err(e) => {
                                error!("{:?}", e);
                                String::from("{}")
                            }
                        },
                        Err(e) => {
                            error!("{:?}", e);
                            String::from("{}")
                        }
                    }
                }
            }
        };
        match rocket::serde::json::from_str::<Self>(&raw_file) {
            Ok(json) => json,
            Err(e) => {
                error!("{:?}", e);
                Self::empty()
            }
        }
    }
}

#[rocket::async_trait]
pub trait ToJsonFile: FileDir + IdGenerator + Serialize + Selfaware {
    async fn to_file(&self, path: String, stringified: String) -> Result<String, VoteErrorKind> {
        let created_artifact = match File::open(&path).await {
            Ok(_f) => Err(VoteErrorKind::Conflict(self.get_type() + " already exist.")),
            Err(_) => self.create(path, stringified).await,
        };
        match created_artifact {
            Ok(_) => Ok(self.update_index().await?),
            Err(e) => Err(e),
        }
    }

    #[cfg(test)]
    async fn create(&self, _path: String, _stringified: String) -> Result<String, VoteErrorKind> {
        Ok(String::from("Done"))
    }

    #[cfg(not(test))]
    async fn create(&self, path: String, stringified: String) -> Result<String, VoteErrorKind> {
        match File::create(&path).await {
            Ok(f) => self.write_all(f, stringified).await,
            Err(e) => {
                error!("{:?}", e);
                Err(VoteErrorKind::IO(e))
            }
        }
    }
    async fn write_all(&self, mut f: File, stringified: String) -> Result<String, VoteErrorKind> {
        match f.write_all(stringified.as_bytes()).await {
            Ok(_) => {
                f.sync_all().await?;
                Ok(String::from("Done"))
            }
            Err(e) => {
                error!("{:?}", e);
                Err(VoteErrorKind::IO(e))
            }
        }
    }
    #[cfg(test)]
    async fn update_index(&self) -> Result<String, VoteErrorKind> {
        Ok(String::from("Saved and index updated."))
    }

    #[cfg(not(test))]
    async fn update_index(&self) -> Result<String, VoteErrorKind> {
        let index_file_path = self.get_full_path(true) + "/index.json";
        match File::open(&index_file_path).await {
            Ok(f) => {
                let mut contents = String::new();
                f.into_std().await.read_to_string(&mut contents).unwrap();
                match rocket::serde::json::from_str::<Vec<String>>(&contents) {
                    Ok(mut vec) => {
                        vec.push(self.get_id());
                        match self
                            .create(
                                index_file_path.clone(),
                                rocket::serde::json::to_string(&vec).unwrap(),
                            )
                            .await
                        {
                            Ok(_) => Ok(String::from("Saved and index updated.")),
                            Err(_) => self.init_index(&index_file_path).await,
                        }
                    }
                    Err(_e) => self.init_index(&index_file_path).await,
                }
            }
            Err(_) => self.init_index(&index_file_path).await,
        }
    }

    #[cfg(test)]
    async fn init_index(&self) -> Result<String, VoteErrorKind> {
        Ok(String::from("Saved and index updated."))
    }
    #[cfg(not(test))]
    async fn init_index(&self, index_file_path: &str) -> Result<String, VoteErrorKind> {
        match self
            .create(
                String::from(index_file_path),
                "[\"".to_owned() + &self.get_id() + "\"]",
            )
            .await
        {
            Ok(_) => Ok(String::from("Saved and index updated.")),
            Err(e) => Err(e),
        }
    }
    async fn to_json_file(&self) -> Result<String, VoteErrorKind> {
        match rocket::serde::json::to_string(&self) {
            Ok(stringified) => {
                let path = self.get_full_path(true) + "/" + &self.get_id() + ".json";
                self.to_file(path, stringified).await
            }
            Err(e) => {
                error!("{:?}", e);
                Err(VoteErrorKind::Serialize(e))
            }
        }
    }
    async fn update_json_file(&self) -> Result<String, VoteErrorKind> {
        match rocket::serde::json::to_string(&self) {
            Ok(stringified) => {
                let path = self.get_full_path(true) + "/" + &self.get_id() + ".json";
                self.create(path, stringified).await
            }
            Err(e) => {
                error!("{:?}", e);
                Err(VoteErrorKind::Serialize(e))
            }
        }
    }
}

#[rocket::async_trait]
pub trait Fill: Empty + FromJsonFile {
    async fn fill(name: &str, internal: bool) -> Self
    where
        Self: Sync + Sized + for<'de> Deserialize<'de>,
    {
        let lowercase_name = name.to_string().to_lowercase();
        Self::empty()
            .from_json_file_to_self(&lowercase_name, internal)
            .await
    }
    async fn fill_json(name: &str, internal: bool) -> Json<Self>
    where
        Self: Sync + Sized + for<'de> Deserialize<'de>,
    {
        let lowercase_name = name.to_string().to_lowercase();
        Json(Self::fill(&lowercase_name, internal).await)
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
/////////////////////////////////////////////
//                                         //
//        VOTE-ERROR-KIND                  //
//                                         //
/////////////////////////////////////////////

#[derive(Debug)]
pub enum VoteErrorKind<'r> {
    MissingField(MissingField<'r>),
    Serialize(rocket::serde::json::serde_json::Error),
    IO(std::io::Error),
    Conflict(String),
    NotFound(String),
    Unauthorized(Unauthorized<String>),
}

impl<'r> ToString for VoteErrorKind<'r> {
    fn to_string(&self) -> String {
        match self {
            VoteErrorKind::Serialize(e) => e.to_string(),
            VoteErrorKind::MissingField(e) => e.to_string(),
            VoteErrorKind::IO(e) => e.to_string(),
            VoteErrorKind::Conflict(e) => e.to_string(),
            VoteErrorKind::NotFound(e) => e.to_string(),
            VoteErrorKind::Unauthorized(e) => e.0.clone(),
        }
    }
}

impl<'r> From<std::io::Error> for VoteErrorKind<'r> {
    fn from(error: std::io::Error) -> Self {
        VoteErrorKind::IO(error)
    }
}

impl<'r> From<rocket::serde::json::serde_json::Error> for VoteErrorKind<'r> {
    fn from(error: rocket::serde::json::serde_json::Error) -> Self {
        VoteErrorKind::Serialize(error)
    }
}

#[derive(Responder, Debug)]
#[response(content_type = "application/json")]
pub struct MissingField<'r> {
    field: &'r str,
}
impl<'r> ToString for MissingField<'r> {
    fn to_string(&self) -> String {
        self.field.to_string()
    }
}

trait ValidFileName {
    fn valid_file_name(self) -> bool;
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

#[derive(Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Votings {
    pub votings: Vec<Voting>,
}

impl FromJsonFile for Votings {}
impl Index for Votings {}
impl FileDir for Votings {
    fn get_dir() -> &'static str {
        "votings"
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
impl Selfaware for Voting {
    fn get_type(&self) -> String {
        String::from("voting")
    }
}
impl IdGenerator for Voting {
    fn get_id(&self) -> String {
        self.generate_id()
    }
    fn generate_id(&self) -> String {
        format!("{}", self.name)
    }
}

/////////////////////////////////////////////
//                                         //
//                 VOTING                  //
//                                         //
/////////////////////////////////////////////
#[derive(Debug, Serialize, PartialEq, Deserialize)]
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
#[derive(Debug, Serialize, PartialEq, Deserialize)]
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
impl FromJsonFile for Voting {}
impl FileDir for Voting {
    fn get_dir() -> &'static str {
        "votings"
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
impl ToJsonFile for Voting {}

/////////////////////////////////////////////
//                                         //
//              CANDIDATE                  //
//                                         //
/////////////////////////////////////////////

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NonVoter<'r>(&'r str);

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Candidate {
    pub voter: bool,
    pub id: Option<String>,
    pub label: String,
}

impl Candidate {
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

impl FileDir for Candidate {
    fn get_dir() -> &'static str {
        "candidates"
    }
}

impl Selfaware for Candidate {
    fn get_type(&self) -> String {
        match self.voter {
            false => String::from("Candidate"),
            true => String::from("Voter"),
        }
    }
}
impl ToJsonFile for Candidate {}
impl Index for Candidate {}
impl IsUnique for Candidate {}
impl FromJsonFile for Candidate {}
#[rocket::async_trait]
impl Fill for Candidate {
    async fn fill(name: &str, internal: bool) -> Self
    where
        Self: Sync + Sized + for<'de> Deserialize<'de>,
    {
        Self::empty()
            .from_json_file_to_self(
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
/////////////////////////////////////////////

#[derive(Serialize, Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CastBallots {
    pub voting: Option<String>,
    pub ballots: Vec<KnownBallots>,
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

impl FileDir for CastBallots {
    fn get_dir() -> &'static str {
        "ballots"
    }
    fn get_full_path(&self, _possible_remote: bool) -> String {
        let root_path = match std::env::var(FILE_DIR) {
            Ok(file_dir) => file_dir,
            Err(_) => String::from("/tmp/"),
        };
        root_path.to_owned() + &Self::get_dir()
    }
}

impl Selfaware for CastBallots {
    fn get_type(&self) -> String {
        String::from("CBallots")
    }
}

impl FromJsonFile for CastBallots {}

impl ToJsonFile for CastBallots {}

impl Fill for CastBallots {}

impl Index for CastBallots {}

/////////////////////////////////////////////
//                                         //
//           KNOWN BALLOT                  //
//                                         //
/////////////////////////////////////////////

#[derive(Serialize, Clone, Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct KnownBallots {
    pub voter: String,
    pub ballots: Vec<Ballot>,
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

#[derive(Clone, Serialize, Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Ballot {
    pub candidate: String,
    pub votes: Vec<Vote>,
    pub notes: Option<String>,
    pub voted_on: Option<DateTime<Utc>>,
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
impl Index for Criteria {}
impl FromJsonFile for Criteria {}

impl FileDir for Criteria {
    fn get_dir() -> &'static str {
        "criteria"
    }
}
#[derive(Serialize, Clone, Debug, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Criterion {
    pub name: String,
    pub min: i8,
    pub max: i8,
    pub weight: Option<f32>,
}
impl Fill for Criterion {}
impl Selfaware for Criterion {
    fn get_type(&self) -> String {
        String::from("Criterion")
    }
}
impl ToJsonFile for Criterion {}
impl FromJsonFile for Criterion {}
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
            self.name.to_owned(),
            self.min.to_string(),
            self.max.to_string(),
            weight,
        ]
        .join("_")
    }
}

impl FileDir for Criterion {
    fn get_dir() -> &'static str {
        "criteria"
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
impl Empty for EmojiCategories {
    fn empty() -> Self {
        Self { emojis: vec![] }
    }
}

impl FileDir for EmojiCategories {
    fn get_dir() -> &'static str {
        "static"
    }
    fn get_full_path(&self, _possible_remote: bool) -> String {
        let asset_dir = match std::env::var(ASSET_DIR) {
            Ok(file_dir) => file_dir,
            Err(_) => String::from("/tmp/"),
        };
        asset_dir + Self::get_dir()
    }
}

impl FromJsonFile for EmojiCategories {}
impl ToJsonFile for EmojiCategories {}
#[derive(Debug, Deserialize, Serialize)]
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
}
