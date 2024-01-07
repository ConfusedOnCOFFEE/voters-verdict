#[cfg(feature = "file")]
use crate::{common::IdGenerator, local::ToFile};
use crate::{
    common::{
        Candidate, CastBallots, Criteria, Criterion, EmojiCategories, Empty, QueryableExt, Voting,
        Votings,
    },
    error::VoteErrorKind,
    persistence::{FromPersistence, Path, ToPersistence},
};
use rocket::{
    error, info,
    serde::{json::Json, Deserialize, Serialize},
};
use std::collections::BTreeMap;

#[cfg(feature = "sqlx_sqlite")]
use crate::db::common::Query;
#[cfg(feature = "file")]
#[rocket::async_trait]
pub trait FromStorage: FromPersistence + Empty {
    async fn load(&self, id: &str, internal: bool) -> Json<Self>
    where
        Self: Sized + for<'de> Deserialize<'de>,
    {
        Json(self.load_into(id, internal).await)
    }

    async fn load_into(&self, id: &str, internal: bool) -> Self
    where
        Self: Sized + for<'de> Deserialize<'de>,
    {
        let path = Path::get_full_path(self, true) + "/" + id + ".json";
        let raw_file = self.from_persistence(&path, internal).await;
        match rocket::serde::json::from_str::<Self>(&raw_file) {
            Ok(json) => json,
            Err(e) => {
                error!("{:?}", e);
                Self::empty()
            }
        }
    }
}
#[cfg(feature = "sqlx_sqlite")]
#[rocket::async_trait]
pub trait FromStorage:
    crate::common::Table + FromPersistence + Empty + Query + std::str::FromStr + std::fmt::Debug
{
    async fn load(&self, id: &str, internal: bool) -> Json<Self>
    where
        Self: Sized + for<'de> Deserialize<'de>,
    {
        Json(self.load_into(id, internal).await)
    }

    async fn load_into(&self, id: &str, _internal: bool) -> Self {
        match crate::db::sqlx_sqlite::select(
            &Self::get_table(false),
            &Self::get_db_columns(),
            &Self::get_identity_column_name(),
            id,
        )
        .await
        {
            Ok(k) => {
                let mut r = String::new();
                k.iter().for_each(|e| r.push_str(e));
                match Self::from_str(&r) {
                    Ok(ok) => ok,
                    Err(_er) => Self::empty(),
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Self::empty()
            }
        }
    }
}
#[cfg(feature = "file")]
#[rocket::async_trait]
pub trait ToStorage: ToPersistence + Serialize + IdGenerator + ToFile {
    async fn save(&self) -> Result<String, VoteErrorKind> {
        match rocket::serde::json::to_string(&self) {
            Ok(stringified) => {
                let path = Path::get_full_path(self, true) + "/" + &self.get_id() + ".json";
                self.save_to_file(path, stringified).await
            }
            Err(e) => {
                error!("{:?}", e);
                Err(VoteErrorKind::Serialize(e))
            }
        }
    }
    async fn update(&self) -> Result<String, VoteErrorKind> {
        match rocket::serde::json::to_string(&self) {
            Ok(stringified) => {
                let path = Path::get_full_path(self, true) + "/" + &self.get_id() + ".json";
                self.create(path, stringified).await
            }
            Err(e) => {
                error!("{:?}", e);
                Err(VoteErrorKind::Serialize(e))
            }
        }
    }
}
#[cfg(feature = "db")]
#[rocket::async_trait]
pub trait ToStorage: ToPersistence + Serialize {
    async fn save(&self) -> Result<String, VoteErrorKind> {
        <Self as Query>::save(&self).await
    }
    async fn update(&self, new_pairs: BTreeMap<&str, &String>) -> Result<String, VoteErrorKind> {
        info!("Update with these values: {:?}", new_pairs);
        if new_pairs.len() == 0 {
            self.create().await
        } else {
            <Self as Query>::update(&self, new_pairs).await
        }
    }
}

impl ToPersistence for Votings {}
impl FromPersistence for Votings {}
impl FromStorage for Votings {}

impl FromStorage for Voting {}
impl ToStorage for Voting {}

impl FromStorage for CastBallots {}
impl ToStorage for CastBallots {}
impl ToPersistence for CastBallots {}

impl ToStorage for Candidate {}
impl FromStorage for Candidate {}

impl FromStorage for Criteria {}
impl FromPersistence for Criteria {}
impl ToPersistence for Criteria {}

impl ToStorage for EmojiCategories {}
impl FromStorage for EmojiCategories {}
impl ToPersistence for EmojiCategories {}
impl FromPersistence for EmojiCategories {}

impl ToPersistence for Criterion {}
impl FromPersistence for Criterion {}
impl ToStorage for Criterion {}
impl FromStorage for Criterion {}

impl ToPersistence for Candidate {}
impl FromPersistence for Candidate {}

impl FromPersistence for CastBallots {}

impl ToPersistence for Voting {}
impl FromPersistence for Voting {}

impl Path for Votings {}
impl Path for Voting {}
impl Path for CastBallots {}
impl Path for Candidate {}
impl Path for Criteria {}
impl Path for Criterion {}
impl Path for EmojiCategories {}
