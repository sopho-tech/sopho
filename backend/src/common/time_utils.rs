use chrono::{DateTime, Duration as ChronoDuration, Utc};
use time::{Duration, OffsetDateTime};

/// Returns the current UTC time as `DateTime<Utc>`
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Returns the current UTC time as `OffsetDateTime`
pub fn now_utc_offset() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// Returns the current UTC time converted to a SeaORM DateTime type
pub fn now_utc_into<T>() -> T
where
    T: From<DateTime<Utc>>,
{
    Utc::now().into()
}

/// Adds the specified number of days to the current UTC time
pub fn now_plus_days(days: i64) -> DateTime<Utc> {
    Utc::now() + ChronoDuration::days(days)
}

/// Adds the specified number of hours to the current UTC time
pub fn now_plus_hours(hours: i64) -> DateTime<Utc> {
    Utc::now() + ChronoDuration::hours(hours)
}

/// Adds the specified number of days to the current UTC time and converts to SeaORM type
pub fn now_plus_days_into<T>(days: i64) -> T
where
    T: From<DateTime<Utc>>,
{
    (Utc::now() + ChronoDuration::days(days)).into()
}

/// Adds the specified number of hours to the current UTC time and converts to SeaORM type
pub fn now_plus_hours_into<T>(hours: i64) -> T
where
    T: From<DateTime<Utc>>,
{
    (Utc::now() + ChronoDuration::hours(hours)).into()
}

/// Converts a `DateTime<Utc>` to `OffsetDateTime` for cookie expiration
/// Falls back to current time + 1 hour if conversion fails (should be rare)
pub fn datetime_to_offset_datetime(dt: &DateTime<Utc>) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(dt.timestamp())
        .unwrap_or_else(|_| now_utc_offset() + Duration::hours(1))
}

/// Adds the specified number of hours to the current UTC time as `OffsetDateTime`
pub fn now_utc_offset_plus_hours(hours: i64) -> OffsetDateTime {
    OffsetDateTime::now_utc() + Duration::hours(hours)
}

/// Adds the specified number of days to the current UTC time as `OffsetDateTime`
pub fn now_utc_offset_plus_days(days: i64) -> OffsetDateTime {
    OffsetDateTime::now_utc() + Duration::days(days)
}

/// Creates a zero-duration `Duration` for cookie expiration (used to expire cookies)
pub fn zero_duration_seconds() -> Duration {
    Duration::seconds(0)
}

/// Checks if a `DateTime<Utc>` is in the past (expired)
pub fn is_expired(dt: &DateTime<Utc>) -> bool {
    dt < &Utc::now()
}
