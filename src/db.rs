use log::info;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::error::Error;

#[derive(Serialize)]
pub struct FetchedMessage {
    pub id: i32,
    pub timestamp: String,
    pub message: String,
}

pub async fn db_init(db_url: &str) -> Result<PgPool, Box<dyn Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    Ok(pool)
}

pub async fn create_table(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let query = "create table if not exists secret(
        id serial primary key,
        timestamp text not null,
        message text not null
    )";

    sqlx::query(query).execute(pool).await?;

    info!(
        "db.rs  - Connection to supabase database is successful and table created if not existed"
    );
    println!(
        "db.rs - Connection to supabase database is successful and table created if not existed"
    );

    Ok(())
}

pub async fn insert_data(
    pool: &PgPool,
    timestamp: &str,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    sqlx::query("INSERT into secret (timestamp,message) values ($1,$2)")
        .bind(timestamp)
        .bind(message)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn fetch_all(pool: &PgPool) -> Result<Vec<FetchedMessage>, Box<dyn Error>> {
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

pub async fn delete_mesg(pool: &PgPool, id: i32) -> Result<(), Box<dyn Error>> {
    let query = "delete from secret where id = $1";

    sqlx::query(query).bind(id).execute(pool).await?;
    Ok(())
}

/*pub async fn delete_all(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let query = "
        DELETE FROM secret;
        DELETE FROM sqlite_sequence WHERE name='secret';
    ";
    sqlx::query(query).execute(pool).await?;

    Ok(())
}*/
