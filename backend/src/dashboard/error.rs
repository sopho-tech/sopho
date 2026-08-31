use crate::authorization::errors::AuthorizationError;
use crate::common::errors::UpdateDashboardError;
use crate::dashboard::constants::{CANVAS_NOT_FOUND_MESSAGE, DASHBOARD_NOT_FOUND_MESSAGE};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{DASHBOARD_NOT_FOUND_MESSAGE}")]
    NotFound,
    #[error("{CANVAS_NOT_FOUND_MESSAGE}")]
    CanvasNotFound,
    #[error("Dashboard can hold at most {0} charts")]
    TooManyCharts(usize),
    #[error("{0}")]
    Internal(String),
}

impl From<UpdateDashboardError> for DashboardError {
    fn from(error: UpdateDashboardError) -> Self {
        match error {
            UpdateDashboardError::TooManyCharts(maximum) => DashboardError::TooManyCharts(maximum),
            UpdateDashboardError::Repository(sea_orm::DbErr::RecordNotFound(_)) => {
                DashboardError::NotFound
            }
            other => DashboardError::Internal(other.to_string()),
        }
    }
}
