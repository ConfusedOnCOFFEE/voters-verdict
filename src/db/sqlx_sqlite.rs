use crate::{
    common::{cleanup_delimiter, DELIMITER},
    error::{FromErrorKind, VoteErrorKind},
    persistence::PersistenceMode,
};
use futures::future::TryFutureExt;
use rocket::{debug, error, info, warn};
use rocket_db_pools::{
    sqlx,
    sqlx::{sqlite::SqliteRow, Column, Row},
};
use std::collections::BTreeMap;

fn get_columns(column_string: &str) -> Vec<&str> {
    column_string.split(',').collect()
}
pub async fn is_unique(
    table: &str,
    identity_column: &str,
    name: &str,
) -> Result<bool, VoteErrorKind<'static>> {
    let database_url = PersistenceMode::to_conform_path();
    let pool = sqlx::SqlitePool::connect(&database_url.to_string()).await?;
    let sql_string = format!(
        "SELECT {}  FROM {} WHERE {} = '{}'",
        identity_column, table, identity_column, name
    );
    info!("IS_UNIQUE SQL: {:?}", sql_string);
    match sqlx::query(&sql_string).fetch_all(&pool).await {
        Ok(r) => {
            if r.len() == 1 {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Err(_e) => Ok(true),
    }
}

fn get_column_value_by_column_name(table: &str, row: &SqliteRow, column_name: &str) -> String {
    let value = match column_name {
        "min" | "max" | "sum" => row.get::<i16, &str>(column_name).to_string(),
        "weight" | "weighted" | "mean" => row.get::<f32, &str>(column_name).to_string(),
        "voter" => match table {
            "candidates" => row.get::<bool, &str>(column_name).to_string(),
            _ => row.get::<String, &str>(column_name).to_string(),
        },
        _ => row.get::<String, &str>(column_name).to_string(),
    };
    value.to_owned() + DELIMITER
}

// UPDATE table_name
// SET column1 = value1, column2 = value2, ...
// WHERE condition;
pub async fn update(
    table: &str,
    new_pairs: BTreeMap<&str, &String>,
    identity_column: &str,
    name: &str,
) -> Result<bool, VoteErrorKind<'static>> {
    let database_url = PersistenceMode::to_conform_path();
    let pool = sqlx::SqlitePool::connect(&database_url.to_string()).await?;
    let cleanup_name = match name.split_once("_") {
        Some((a, _b)) => match table {
            "criteria" => a,
            "candidates" => name,
            _ => a,
        },
        None => name,
    };
    let sql_string = format!(
        "UPDATE {} SET {} WHERE {} = '{}'",
        table,
        new_pairs
            .iter()
            .map(|(k, v)| format!("{} = '{}',", k, v))
            .collect::<String>()
            .strip_suffix(",")
            .unwrap(),
        identity_column,
        cleanup_name
    );
    info!("UPDATE SQL: {:?}", sql_string);
    let rows = sqlx::query(&sql_string)
        .execute(&pool)
        .await?
        .rows_affected();

    if rows > 0 {
        Ok(true)
    } else {
        Err(FromErrorKind::MultipleRows(true).into())
    }
}

pub async fn select(
    table: &str,
    object_columns: &str,
    identity_column: &str,
    name: &str,
) -> Result<Vec<String>, VoteErrorKind<'static>> {
    let database_url = PersistenceMode::to_conform_path();
    let pool = sqlx::SqlitePool::connect(&database_url.to_string()).await?;
    let cleanup_name = match name.split_once("_") {
        Some((a, _b)) => match table {
            "criteria" => a,
            "candidates" => name,
            _ => a,
        },
        None => name,
    };
    let sql_string = format!(
        "SELECT {} FROM {} WHERE {} = '{}'",
        object_columns, table, identity_column, cleanup_name
    );
    info!("SELECT SQL: {:?}", sql_string);
    match sqlx::query(&sql_string)
        .fetch_one(&pool)
        .map_ok(|r| {
            info!("{:?}", r.columns());
            let mut result: Vec<String> = Vec::new();
            for (i, c) in r.columns().iter().enumerate() {
                info!("{}, {}", i, c.name());
                result.push(get_column_value_by_column_name(table, &r, c.name()));
            }
            Ok(result)
        })
        .await
    {
        Ok(r) => {
            info!("Selecting one worked");
            r
        }
        Err(e) => {
            warn!("Selecting one failed: {}", e);
            Ok(Vec::new())
        }
    }
}

pub async fn list_rows(
    table: &str,
    object_columns: &str,
    column_name: &str,
    all: bool,
) -> Result<Vec<String>, VoteErrorKind<'static>> {
    let database_url = PersistenceMode::to_conform_path();
    let pool = sqlx::SqlitePool::connect(&database_url.to_string()).await?;
    let sql_string = format!("SELECT {} FROM {} ", object_columns, table);
    info!("SELECT_ALL SQL: {}", sql_string);
    let rows = sqlx::query(&sql_string).fetch_all(&pool).await?;
    if rows.len() >= 1 {
        let mut result: Vec<String> = Vec::new();
        for (_idx, r) in rows.iter().enumerate() {
            let mut next_row = String::new();
            if all {
                get_columns(object_columns).iter().for_each(|c| {
                    let a =
                        cleanup_delimiter(&get_column_value_by_column_name(table, &r, c.trim()))
                            .replace(",", "_");
                    next_row.push_str(&a)
                });
            } else {
                next_row.push_str(&cleanup_delimiter(&get_column_value_by_column_name(
                    table,
                    &r,
                    column_name,
                )));
            }
            match next_row.strip_suffix('_') {
                Some(b) => next_row = b.to_string(),
                None => {}
            };

            match next_row.strip_suffix(',') {
                Some(b) => next_row = b.to_string(),
                None => {}
            };
            result.push(next_row);
        }
        debug!("Extracted list_rows: {:?}", result);
        Ok(result)
    } else {
        Ok(vec![])
    }
}
pub async fn save(table: &str, object_row: &str) -> Result<String, VoteErrorKind<'static>> {
    let database_url = PersistenceMode::to_conform_path();
    let pool = sqlx::SqlitePool::connect(&database_url.to_string()).await?;
    let sql_string = format!(
        "INSERT into {} VALUES ({})",
        table,
        cleanup_delimiter(object_row)
    );
    info!("INERT SQL: {}", sql_string);
    match sqlx::query(&sql_string).execute(&pool).await {
        Ok(_) => Ok(String::from(object_row)),
        Err(e) => {
            error!("{:?}", e);
            Err(VoteErrorKind::DB(e))
        }
    }
}
