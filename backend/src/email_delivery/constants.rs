pub const RETRY_DELAYS_MINUTES: [i64; 5] = [1, 5, 30, 120, 360];
pub const SENDING_LEASE_MINUTES: i64 = 5;
pub const SENDER_POLL_INTERVAL_SECONDS: u64 = 5;
pub const SENDER_BATCH_SIZE: u64 = 50;
pub const NOTICE_SEND_WINDOW_HOURS: i64 = 24;
pub const UNDECRYPTABLE_EMAIL_MESSAGE: &str = "The email could not be decrypted";
pub const EXPIRED_EMAIL_MESSAGE: &str = "Its links expired before it could be sent";
pub const SMTP_TIMED_OUT_MESSAGE: &str = "The mail server did not answer in time";
pub const EMAIL_DISABLED_MESSAGE: &str = "Email is not configured";
pub const EMAIL_NOT_FOUND_MESSAGE: &str = "Email not found";
pub const EMAIL_NOT_RETRYABLE_MESSAGE: &str =
    "Only a failed email whose links are still valid can be retried";

pub use sopho_common::email_status::EmailStatus;
