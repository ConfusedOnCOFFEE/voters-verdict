#[cfg(feature = "diesel_sqlite")]
use super::sqlite::{establish_connection, query_voting, EmptySelectable};
use crate::{
    common::{
        Candidate, CastBallots, Criteria, Criterion, EmojiCategories, Empty, QueryableExt, Voting,
        Votings,
    },
    error::VoteErrorKind,
};
use rocket::{debug, error};
use std::collections::BTreeMap;
#[rocket::async_trait]
pub trait Query: QueryableExt + std::str::FromStr + std::marker::Sync {
    fn get_dir() -> String {
        debug!("db::query::get_dir");
        String::from("dummy")
    }
    async fn is_unique(&self, id: &str) -> Result<String, VoteErrorKind> {
        debug!("db::query::is_inque");
        match crate::db::sqlx_sqlite::is_unique(
            &Self::get_table(false),
            &Self::get_identity_column_name(),
            id,
        )
        .await
        {
            Ok(k) => {
                if k {
                    Ok(String::from(id))
                } else {
                    Err(VoteErrorKind::NotFound(String::from("Not found")))
                }
            }
            Err(_) => Err(VoteErrorKind::NotFound(String::from("Not found"))),
        }
    }
    async fn index(&self) -> Result<Vec<String>, String> {
        debug!("db::query::index");
        self.list_rows(false).await
    }

    #[cfg(feature = "diesel_sqlite")]
    async fn select<T: EmptySelectable>(&self, id: &str) -> Result<T, String> {
        let conn = establish_connection();
        match query_voting(&mut conn, id).await {
            Ok(r) => Ok(r),
            Err(_) => Err(String::from("No hit.")),
        }
    }
    #[cfg(feature = "sqlx_sqlite")]
    async fn list_rows(&self, all: bool) -> Result<Vec<String>, String> {
        debug!("db::query::list_rows");
        match crate::db::sqlx_sqlite::list_rows(
            &Self::get_table(false),
            &Self::get_db_columns(),
            &Self::get_identity_column_name(),
            all,
        )
        .await
        {
            Ok(k) => Ok(k),
            Err(e) => {
                error!("{:?}", e);
                Err(String::from("No entries"))
            }
        }
    }
    async fn create(&self) -> Result<String, VoteErrorKind> {
        debug!("db::query::create");
        self.save().await
    }

    async fn update_index(&self) -> Result<String, VoteErrorKind> {
        debug!("db::query::unpdate_index");
        self.save().await
    }

    #[cfg(feature = "sqlx_sqlite")]
    async fn update(&self, new_pairs: BTreeMap<&str, &String>) -> Result<String, VoteErrorKind> {
        debug!("db::query::update");
        match crate::db::sqlx_sqlite::update(
            &Self::get_table(true),
            new_pairs,
            &Self::get_identity_column_name(),
            &self.get_name(),
        )
        .await
        {
            Ok(_) => Ok(self.get_id()),
            Err(er) => {
                error!("{:?}", er);
                Err(er)
            }
        }
    }
    #[cfg(feature = "sqlx_sqlite")]
    async fn save(&self) -> Result<String, VoteErrorKind> {
        debug!("db::query::save");
        let found = crate::db::sqlx_sqlite::select(
            &Self::get_table(false),
            &Self::get_db_columns(),
            &Self::get_identity_column_name(),
            &self.get_name(),
        )
        .await?;
        if found.is_empty() {
            match crate::db::sqlx_sqlite::save(&Self::get_table(true), &self.to_db_row()).await {
                Ok(_) => Ok(self.get_id()),
                Err(er) => {
                    error!("{:?}", er);
                    Err(er)
                }
            }
        } else {
            Err(VoteErrorKind::Conflict(String::from("Already exist.")))
        }
    }
}

impl QueryableExt for Votings {}
impl QueryableExt for Voting {}
impl QueryableExt for CastBallots {}
impl QueryableExt for Candidate {}
impl QueryableExt for Criteria {}
impl QueryableExt for Criterion {}
impl QueryableExt for EmojiCategories {}

impl Query for Votings {}
impl Query for Voting {}
impl Query for CastBallots {}
impl Query for Candidate {}
impl Query for Criteria {}
impl Query for Criterion {}
impl Query for EmojiCategories {}
