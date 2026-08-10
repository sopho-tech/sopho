use sqlx::postgres::PgConnection;
use sqlx::sqlite::SqliteConnection;

#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<serde_json::Value>,
    pub data: Vec<serde_json::Value>,
}

pub enum DatabaseConnection {
    Postgres(PgConnection),
    Sqlite(SqliteConnection),
}
