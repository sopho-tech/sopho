use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_pages: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_items: u64,
}
