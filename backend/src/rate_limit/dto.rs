use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct RateLimitQuota {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct RateLimitSettings {
    pub sign_in: RateLimitQuota,
    pub api: RateLimitQuota,
    pub ai: RateLimitQuota,
}
