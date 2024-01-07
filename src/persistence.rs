use crate::{
    common::{IdGenerator, Selfaware},
    config::STORAGE_MODE,
    error::VoteErrorKind,
};
#[cfg(feature = "local")]
use rocket::error;
use rocket::{debug, serde::Deserialize};

#[cfg(feature = "local")]
use crate::{
    config::FILE_DIR,
    local::{DirectoryIndex, FileDir, IsUniqueFile, ToFile},
};
#[cfg(feature = "remote")]
use crate::{config::REMOTE_STORAGE, remote::RemoteIndex};
#[cfg(feature = "db")]
use crate::{config::SQLITE_CONNECTION, db::common::Query};

pub enum PersistenceMode {
    #[cfg(feature = "db")]
    DB(DatabaseConfig),
    #[cfg(feature = "local")]
    File(FileConfig),
    #[cfg(feature = "remote")]
    Remote(RemoteConfig),
}
#[cfg(feature = "remote")]
pub struct RemoteConfig {
    url: Option<String>,
}
#[cfg(feature = "local")]
pub struct FileConfig {
    dir: String,
}

#[cfg(feature = "db")]
pub struct DatabaseConfig {
    url: Option<String>,
}

impl PersistenceMode {
    pub fn detect_config_mode() -> Self {
        match std::env::var(STORAGE_MODE) {
            Ok(s) => PersistenceMode::from(s.as_str()),
            Err(_) => PersistenceMode::default_mode(),
        }
    }
    #[cfg(feature = "local")]
    fn default_mode() -> Self {
        PersistenceMode::File(FileConfig {
            dir: match std::env::var(FILE_DIR) {
                Ok(file_dir) => file_dir,
                Err(_) => String::from("/tmp/"),
            },
        })
    }
    #[cfg(feature = "db")]
    fn default_mode() -> Self {
        PersistenceMode::DB(DatabaseConfig {
            url: match std::env::var(SQLITE_CONNECTION) {
                Ok(connection) => Some(connection),
                Err(_) => panic!("No sqlite connnection backup defined"),
            },
        })
    }
    pub fn to_conform_path() -> String {
        match PersistenceMode::detect_config_mode() {
            #[cfg(feature = "db")]
            PersistenceMode::DB(d) => match d.url {
                Some(s) => s.to_string(),
                None => panic!("No DB"),
            },
            #[cfg(feature = "local")]
            PersistenceMode::File(d) => d.dir,
            #[cfg(feature = "remote")]
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
impl From<&str> for PersistenceMode {
    fn from(mode: &str) -> Self {
        match mode {
            #[cfg(feature = "db")]
            "1" | "db" => PersistenceMode::DB(DatabaseConfig {
                url: match std::env::var(SQLITE_CONNECTION) {
                    Ok(connection) => Some(connection),
                    Err(_) => panic!("No sqlite connnection backup defined"),
                },
            }),
            #[cfg(feature = "local")]
            "2" | "local" => PersistenceMode::File(FileConfig {
                dir: match std::env::var(FILE_DIR) {
                    Ok(file_dir) => file_dir,
                    Err(_) => String::from("/tmp/"),
                },
            }),
            #[cfg(feature = "remote")]
            "3" | "remote" => PersistenceMode::Remote(RemoteConfig {
                url: match std::env::var(REMOTE_STORAGE) {
                    Ok(connection) => Some(connection),
                    Err(_) => panic!("No remote storage defined"),
                },
            }),
            _ => PersistenceMode::default_mode(),
        }
    }
}

#[rocket::async_trait]
#[cfg(all(feature = "local", not(feature = "remote")))]
pub trait Path: FileDir + IsUniqueFile {
    fn get_dir() -> &'static str {
        match PersistenceMode::detect_config_mode() {
            PersistenceMode::File(_d) => <Self as FileDir>::get_dir(),
            _ => panic!("n/a"),
        }
    }
    fn get_full_path(&self, possible_remote: bool) -> String {
        let root_path = match possible_remote {
            true => PersistenceMode::to_conform_path(),
            false => match std::env::var(FILE_DIR) {
                Ok(file_dir) => file_dir,
                Err(_) => String::from("/tmp/"),
            },
        };
        root_path + &<Self as FileDir>::get_dir()
    }

    async fn is_unique(&self, id: &str) -> Result<String, VoteErrorKind> {
        match PersistenceMode::detect_config_mode() {
            PersistenceMode::File(_) => <Self as IsUniqueFile>::is_unique(self, id).await,
        }
    }
}

#[cfg(feature = "file")]
#[rocket::async_trait]
pub trait Path: FileDir + IsUniqueFile {
    fn get_dir() -> &'static str {
        match PersistenceMode::detect_config_mode() {
            #[cfg(feature = "local")]
            PersistenceMode::File(_d) => <Self as FileDir>::get_dir(),
            _ => panic!("n/a"),
        }
    }
    fn get_full_path(&self, possible_remote: bool) -> String {
        let root_path = match possible_remote {
            true => PersistenceMode::to_conform_path(),
            false => match std::env::var(FILE_DIR) {
                Ok(file_dir) => file_dir,
                Err(_) => String::from("/tmp/"),
            },
        };
        root_path + &<Self as FileDir>::get_dir()
    }

    async fn is_unique(&self, id: &str) -> Result<String, VoteErrorKind> {
        match PersistenceMode::detect_config_mode() {
            #[cfg(feature = "local")]
            PersistenceMode::File(_) => <Self as IsUniqueFile>::is_unique(self, id).await,
            PersistenceMode::Remote(_) => Ok(String::from("name")),
        }
    }
}

#[cfg(feature = "db")]
#[rocket::async_trait]
pub trait Path: Query + Selfaware {
    async fn get_dir(&self) -> String {
        self.get_type().to_string()
    }
    fn get_full_path(&self, _possible_remote: bool) -> String {
        <Self as Query>::get_dir()
    }
    async fn is_unique(&self, id: &str) -> Result<String, VoteErrorKind> {
        debug!("is_unique");
        match id.split_once("_") {
            Some((a, _b)) => <Self as Query>::is_unique(self, a).await,
            None => <Self as Query>::is_unique(self, id).await,
        }
    }
}

#[cfg(all(not(feature = "file"), feature = "local"))]
#[rocket::async_trait]
pub trait ToPersistence: DirectoryIndex + Path + ToFile {
    async fn index(&self) -> Result<Vec<String>, String> {
        let uri = Path::get_full_path(self, true) + "/index.json";
        self.read_index(&uri).await
    }

    async fn list(&self) -> Result<Vec<String>, String> {
        self.list_dir().await
    }
}

#[cfg(all(not(feature = "file"), feature = "remove"))]
#[rocket::async_trait]
pub trait ToPersistence: RemoteIndex + Path + ToFile {
    async fn index(&self) -> Result<Vec<String>, String> {
        let uri = Path::get_full_path(self, true) + "/index.json";
        self.prepare(&uri).await
    }
}

#[cfg(feature = "file")]
#[rocket::async_trait]
pub trait ToPersistence: DirectoryIndex + RemoteIndex + Path + ToFile {
    async fn index(&self) -> Result<Vec<String>, String> {
        let uri = FileDir::get_full_path(self, true) + "/index.json";
        match PersistenceMode::detect_config_mode() {
            PersistenceMode::File(_d) => self.read_index(&uri).await,
            PersistenceMode::Remote(_d) => self.prepare(&uri).await,
        }
    }

    async fn list(&self) -> Result<Vec<String>, String> {
        self.list_dir().await
    }
}

#[cfg(feature = "db")]
#[rocket::async_trait]
pub trait ToPersistence: Query + Selfaware + IdGenerator {
    async fn index(&self) -> Result<Vec<String>, String> {
        <Self as Query>::index(self).await
    }

    async fn list(&self) -> Result<Vec<String>, String> {
        <Self as Query>::list_rows(self, true).await
    }
}
#[cfg(feature = "file")]
#[rocket::async_trait]
pub trait FromPersistence: Path + for<'de> Deserialize<'de> {
    async fn from_persistence(&self, path: &str, internal: bool) -> String {
        match PersistenceMode::detect_config_mode() {
            #[cfg(feature = "local")]
            PersistenceMode::File(_d) => self.from_file(path).await,
            #[cfg(feature = "remote")]
            PersistenceMode::Remote(_d) => {
                if internal {
                    self.from_file(&path).await
                } else {
                    match reqwest::get(path).await {
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
        }
    }
}

#[cfg(feature = "db")]
#[rocket::async_trait]
pub trait FromPersistence: Path + for<'de> Deserialize<'de> {
    async fn from_persistence(&self, _path: &str, _internal: bool) -> String {
        match PersistenceMode::detect_config_mode() {
            PersistenceMode::DB(d) => d.url.unwrap().to_string(),
        }
    }
}
