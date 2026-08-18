#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<serde_json::Value>,
    pub data: Vec<serde_json::Value>,
}

pub const POSTGRES_POOL_MAX_CONNECTIONS: u32 = 10;
pub const SQLITE_POOL_MAX_CONNECTIONS: u32 = 4;
pub const CATALOG_SAMPLE_VALUES_PER_COLUMN: usize = 5;
pub const CATALOG_SAMPLE_COLUMNS_PER_QUERY: usize = 50;
pub const CATALOG_TABLE_FETCH_CONCURRENCY: usize = 8;
