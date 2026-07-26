#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Canvas,
    SqlCell,
    ChartCell,
    Conversation,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Canvas => "canvas",
            EntityType::SqlCell => "sql_cells",
            EntityType::ChartCell => "chart_cells",
            EntityType::Conversation => "conversations",
        }
    }
}

pub const SEARCH_LIMIT_PER_ENTITY: u64 = 5;
