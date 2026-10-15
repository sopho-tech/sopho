use crate::common::constants::CreatedByFilter;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize)]
pub struct Pagination {
    page: Option<u64>,
    page_size: Option<u64>,
}

impl Pagination {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1)
    }

    pub fn page_size(&self) -> u64 {
        self.page_size.unwrap_or(10)
    }
}

#[derive(Deserialize)]
pub struct CreatedByQuery {
    created_by: Option<String>,
}

impl CreatedByQuery {
    pub fn filter(&self) -> Result<Option<CreatedByFilter>, strum::ParseError> {
        self.created_by
            .as_deref()
            .map(CreatedByFilter::from_str)
            .transpose()
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_pages: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_items: u64,
}
