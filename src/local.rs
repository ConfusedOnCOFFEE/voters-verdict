use crate::{
    common::{
        Candidate, CastBallots, Criteria, Criterion, EmojiCategories, IdGenerator, Selfaware,
        Voting, Votings,
    },
    config::{ASSET_DIR, FILE_DIR},
    error::VoteErrorKind,
    persistence::Path,
};
use rocket::{
    debug, error,
    serde::Deserialize,
    tokio::{
        fs::File,
        io::{AsyncReadExt, AsyncWriteExt},
    },
};
use std::io::Read;
pub trait FileDir {
    fn get_dir() -> &'static str;
    fn get_full_path(&self, _possible_remote: bool) -> String {
        let root_path = match std::env::var(FILE_DIR) {
            Ok(file_dir) => file_dir,
            Err(_) => String::from("/tmp/"),
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
pub trait DirectoryIndex: FileDir + FromFile + for<'de> Deserialize<'de> {
    async fn list_dir(&self) -> Result<Vec<String>, String> {
        let full_path = FileDir::get_full_path(self, true);
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
    async fn read_index(&self, uri: &str) -> Result<Vec<String>, String> {
        debug!("Reading file from: {:?}", uri);
        let raw_file = self.from_file(uri).await;
        match rocket::serde::json::from_str::<Vec<String>>(&raw_file) {
            Ok(json) => Ok(json),
            Err(_votes) => Ok(vec![]),
        }
    }
}
#[rocket::async_trait]
pub trait IsUniqueFile: DirectoryIndex {
    async fn is_unique(&self, id: &str) -> Result<String, VoteErrorKind>
    where
        Self: Sync,
    {
        match self.list_dir().await {
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
pub trait FromFile {
    async fn from_file(&self, path: &str) -> String
    where
        Self: Sized + for<'de> Deserialize<'de>,
    {
        match File::open(path).await {
            Ok(r) => self.read_to_string(r).await,
            Err(e) => {
                error!("Error: {:?} on {:?}", e, path);
                String::new()
            }
        }
    }
    async fn read_to_string(&self, f: File) -> String {
        let mut contents = String::new();
        f.into_std().await.read_to_string(&mut contents).unwrap();
        contents
    }
}

#[rocket::async_trait]
pub trait ToFile: FileDir + IdGenerator + Selfaware {
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
    async fn save(&self, path: String, stringified: String) -> Result<String, VoteErrorKind> {
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
}

impl DirectoryIndex for Votings {}
impl FileDir for Votings {
    fn get_dir() -> &'static str {
        "votings"
    }
}

impl FromFile for Votings {}

impl FileDir for Voting {
    fn get_dir() -> &'static str {
        "votings"
    }
}
impl ToFile for Votings {}

impl IsUniqueFile for Votings {}
impl ToFile for Voting {}
impl FromFile for Voting {}

impl DirectoryIndex for Voting {}
impl IsUniqueFile for Voting {}

impl FromFile for CastBallots {}

impl ToFile for CastBallots {}
impl IsUniqueFile for CastBallots {}
impl DirectoryIndex for CastBallots {}

impl FileDir for Candidate {
    fn get_dir() -> &'static str {
        "candidates"
    }
}

impl FromFile for Candidate {}

impl ToFile for Candidate {}

impl DirectoryIndex for Candidate {}
impl IsUniqueFile for Candidate {}

impl FileDir for CastBallots {
    fn get_dir() -> &'static str {
        "ballots"
    }
    fn get_full_path(&self, _possible_remote: bool) -> String {
        let root_path = match std::env::var(FILE_DIR) {
            Ok(file_dir) => file_dir,
            Err(_) => String::from("/tmp/"),
        };
        root_path.to_owned() + &<CastBallots as FileDir>::get_dir()
    }
}

impl ToFile for Criteria {}
impl DirectoryIndex for Criteria {}
impl FromFile for Criteria {}

impl IsUniqueFile for Criteria {}
impl FileDir for Criteria {
    fn get_dir() -> &'static str {
        "criteria"
    }
}

impl FromFile for Criterion {}

impl ToFile for Criterion {}

impl IsUniqueFile for Criterion {}
impl DirectoryIndex for Criterion {}
impl FileDir for Criterion {
    fn get_dir() -> &'static str {
        "criteria"
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
        asset_dir + <EmojiCategories as FileDir>::get_dir()
    }
}
impl FromFile for EmojiCategories {}
impl ToFile for EmojiCategories {}

impl IsUniqueFile for EmojiCategories {}
impl DirectoryIndex for EmojiCategories {}
