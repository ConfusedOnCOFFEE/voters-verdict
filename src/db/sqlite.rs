use crate::{
    ballots::BallotsTable,
    common::{Empty, Voting, VotingTable, Votings},
    persistence::PersistenceMode,
    // db::common::QueryableExt
};
use diesel::{Connection, SqliteConnection};
use rocket_sync_db_pools::{database, diesel};
type Result<T, E = rocket::response::Debug<diesel::result::Error>> = std::result::Result<T, E>;
pub trait EmptySelectable: Empty + diesel::Selectable<DB> {}

async fn query_votings(conn: &mut SqliteConnection) -> Result<Votings> {
    use crate::common::votings::dsl::votings;
    use diesel::RunQueryDsl;
    let rows: Vec<VotingTable> = crate::common::votings::table
        .load(conn)
        .expect("Error loading posts");
    Ok(Votings::from(rows))
}
pub async fn query_voting<T: EmptySelectable>(conn: &mut SqliteConnection, id: &str) -> Result<T> {
    use crate::common::votings::dsl::votings;
    use diesel::{OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
    let voting = crate::common::votings::table
        .find(id)
        .select(T::as_select())
        .first(conn);
    match voting {
        Ok(v) => Ok(v),
        Err(_) => Ok(T::empty()),
    }
}

fn query_ballots(conn: &mut SqliteConnection, ballots: crate::common::CastBallots) -> BallotsTable {
    use crate::ballots::ballots::dsl::*;
    use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
    conn.transaction(|conn| {
        diesel::insert_into(crate::ballots::ballots::table)
            .values(crate::ballots::ballots::table::from(ballots))
            .execute(conn)?;

        crate::ballots::ballots::table
            .order(crate::ballots::ballots::voted_on.desc())
            .select(BallotsTable::as_select())
            .first(conn)
    })
    .expect("Error while saving post")
}
pub fn establish_connection() -> SqliteConnection {
    let database_url = PersistenceMode::to_conform_path();
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
