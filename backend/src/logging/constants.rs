use crate::request_trace::constants::{
    JOB_SPAN_FIELD, METHOD_SPAN_FIELD, PATH_SPAN_FIELD, REQUEST_ID_SPAN_FIELD,
    TENANT_ID_SPAN_FIELD, USER_ID_SPAN_FIELD,
};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum LogsFormat {
    Text,
    Json,
}

pub const DEFAULT_LOGS_FORMAT: LogsFormat = LogsFormat::Text;
pub const TIMESTAMP_LOG_FIELD: &str = "timestamp";
pub const LEVEL_LOG_FIELD: &str = "level";
pub const TARGET_LOG_FIELD: &str = "target";
pub const TRACE_ID_LOG_FIELDS: [&str; 6] = [
    REQUEST_ID_SPAN_FIELD,
    METHOD_SPAN_FIELD,
    PATH_SPAN_FIELD,
    USER_ID_SPAN_FIELD,
    TENANT_ID_SPAN_FIELD,
    JOB_SPAN_FIELD,
];
