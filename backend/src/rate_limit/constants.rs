pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 60;
pub const MINIMUM_RATE_LIMIT_VALUE: u32 = 1;
pub const RETRY_AFTER_ROUNDING_SECONDS: u64 = 1;
pub const SINGLE_SECOND: u64 = 1;
pub const SECOND_UNIT: &str = "second";
pub const SECONDS_UNIT: &str = "seconds";
pub const TOO_MANY_REQUESTS_MESSAGE_PREFIX: &str = "Too many requests. Try again in";
pub const IPV6_NETWORK_SEGMENT_COUNT: usize = 4;
pub const LIMITER_CLEANUP_INTERVAL_SECONDS: u64 = 60;
pub const FORWARDED_FOR_HEADER: &str = "x-forwarded-for";
pub const FORWARDED_FOR_ENTRY_SEPARATOR: char = ',';
pub const DEFAULT_SIGN_IN_REQUESTS_PER_MINUTE: u32 = 10;
pub const DEFAULT_SIGN_IN_BURST_SIZE: u32 = 10;
pub const DEFAULT_API_REQUESTS_PER_MINUTE: u32 = 600;
pub const DEFAULT_API_BURST_SIZE: u32 = 120;
pub const DEFAULT_AI_REQUESTS_PER_MINUTE: u32 = 20;
pub const DEFAULT_AI_BURST_SIZE: u32 = 10;
pub const RATE_LIMIT_KEY_UNAVAILABLE_MESSAGE: &str =
    "The request could not be attributed to a client for rate limiting";
pub const MISSING_FORWARDED_ADDRESS_LOG_MESSAGE: &str = "rate limiting trusts the proxy's x-forwarded-for header, but the request has none or its last entry is not an address; the request did not come through the proxy";
pub const MISSING_PEER_ADDRESS_LOG_MESSAGE: &str =
    "rate limiting needs the peer address, but the server was started without connection info";
pub const MISSING_AUTHENTICATED_USER_LOG_MESSAGE: &str =
    "a user rate limit ran on a request the authentication guard did not identify";
