use log::info;
use serde::Serialize;
use sqlx::{Row, SqlitePool};
use std::error::Error;

#[derive(Serialize)]
pub struct FetchedMessage {
    pub id: i64,
    pub timestamp: String,
    pub message: String,
}

pub async fn db_init(db_path: &str) -> Result<SqlitePool, Box<dyn Error>> {
    let pool = match SqlitePool::connect(db_path).await {
        Ok(p) => p,
        Err(e) => panic!("cannot connect to db due to {e}"),
    };

    info!("db.rs  - Connection to database is successful");
    Ok(pool)
}

pub async fn insert_data(
    pool: &SqlitePool,
    timestamp: &str,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    sqlx::query("INSERT into secret (timestamp,message) values (?,?)")
        .bind(timestamp)
        .bind(message)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn fetch_all(pool: &SqlitePool) -> Result<Vec<FetchedMessage>, Box<dyn Error>> {
    let query = "select id,timestamp,message from secret";

    let rows = sqlx::query(query).fetch_all(pool).await?;

    let results = rows
        .iter()
        .map(|row| FetchedMessage {
            id: row.get("id"),
            timestamp: row.get("timestamp"),
            message: row.get("message"),
        })
        .collect();

    info!("db.rs - All messages from database is fetched");
    Ok(results)
}
