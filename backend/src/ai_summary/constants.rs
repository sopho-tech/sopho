use serde::{Deserialize, Serialize};
use std::fmt;

pub const DASHBOARD_TOTAL_ROW_BUDGET: usize = 500;
pub const MIN_ROWS_PER_CHART: usize = 10;
pub const MAX_ROWS_PER_CHART: usize = 50;
pub const CHART_SUMMARY_ROW_LIMIT: usize = 50;
pub const GENERATION_CONCURRENCY: usize = 4;
pub const STALE_GENERATION_TIMEOUT_MINUTES: i64 = 5;
pub const MAX_USER_PROMPT_LENGTH: usize = 500;

pub fn rows_per_chart(chart_count: usize) -> usize {
    if chart_count == 0 {
        return MAX_ROWS_PER_CHART;
    }
    (DASHBOARD_TOTAL_ROW_BUDGET / chart_count).clamp(MIN_ROWS_PER_CHART, MAX_ROWS_PER_CHART)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryEntityType {
    Dashboard,
    ChartCell,
}

impl fmt::Display for SummaryEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SummaryEntityType::Dashboard => "DASHBOARD",
            SummaryEntityType::ChartCell => "CHART_CELL",
        })
    }
}

impl SummaryEntityType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "DASHBOARD" => Ok(SummaryEntityType::Dashboard),
            "CHART_CELL" => Ok(SummaryEntityType::ChartCell),
            _ => Err(format!("Invalid summary entity type: {}", s)),
        }
    }

    pub fn serialize_to_str<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryStatus {
    Idle,
    Generating,
    Ready,
    Failed,
}

impl fmt::Display for SummaryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SummaryStatus::Idle => "IDLE",
            SummaryStatus::Generating => "GENERATING",
            SummaryStatus::Ready => "READY",
            SummaryStatus::Failed => "FAILED",
        })
    }
}

impl SummaryStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "IDLE" => Ok(SummaryStatus::Idle),
            "GENERATING" => Ok(SummaryStatus::Generating),
            "READY" => Ok(SummaryStatus::Ready),
            "FAILED" => Ok(SummaryStatus::Failed),
            _ => Err(format!("Invalid summary status: {}", s)),
        }
    }

    pub fn serialize_to_str<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }
}
