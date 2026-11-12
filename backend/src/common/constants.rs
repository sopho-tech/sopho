use serde::Deserialize;
use strum::{AsRefStr, Display, EnumString};

pub const API_PATH_PREFIX: &str = "/api/";
pub const LINK_TOKEN_BYTES: usize = 32;
pub const SMTP_TIMEOUT_SECONDS: u64 = 30;
pub const OIDC_PROVIDER_VARIABLE_PREFIX: &str = "SOPHO_OIDC_";
pub const DEFAULT_SELF_SERVE_TENANT_LIMIT: u64 = 2;
pub const MINIMUM_SELF_SERVE_TENANT_LIMIT: u64 = 1;
pub const PLATFORM_ADMINISTRATOR_EMAIL_SEPARATOR: char = ',';
pub const NOBODY_CAN_SIGN_IN_REASON: &str =
    "with password sign-in off, people need an OIDC provider to sign in";
pub const SEEDED_ADMINISTRATOR_CANNOT_SIGN_IN_REASON: &str =
    "the seeded administrator has only a password, so it could never sign in";
pub const NO_PLATFORM_ADMINISTRATOR_REASON: &str =
    "without a seeded administrator, name at least one platform administrator email";
pub const ADMINISTRATOR_EMAILS_NEED_PROVIDER_REASON: &str =
    "a listed person becomes an administrator by signing in through an OIDC provider";
pub const DEFAULT_TENANT_NEEDS_ADMINISTRATOR_REASON: &str =
    "the default tenant and its demo are owned by the seeded administrator";
pub const DISABLED_CONNECTION_TYPE_SEPARATOR: char = ',';
pub const DEFAULT_TENANT_DEMO_NEEDS_SQLITE_REASON: &str =
    "the default tenant's demo is a SQLite connection";
pub const CONTENT_TYPE_OPTIONS_HEADER_VALUE: &str = "nosniff";
pub const REFERRER_POLICY_HEADER_VALUE: &str = "strict-origin-when-cross-origin";
pub const CONTENT_SECURITY_POLICY_HEADER_VALUE: &str = "frame-ancestors 'none'";
pub const FRAME_OPTIONS_HEADER_VALUE: &str = "DENY";
pub const UNKNOWN_CREATED_BY_FILTER_MESSAGE: &str = "created_by only accepts \"me\"";

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum CreatedByFilter {
    #[strum(serialize = "me")]
    Me,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvironmentVariable {
    DatabaseUrl,
    Port,
    RustLog,
    SophoFrontendDir,
    SophoEnvironment,
    SophoCookieSecure,
    SophoEncryptionKey,
    SophoAdminUsername,
    SophoAdminPassword,
    SophoAdminEmail,
    SophoAdminFullName,
    SophoAnthropicBaseUrl,
    SophoOpenaiBaseUrl,
    SophoOidcProviders,
    SophoSmtpHost,
    SophoSmtpPort,
    SophoSmtpTls,
    SophoSmtpUsername,
    SophoSmtpPassword,
    SophoEmailFrom,
    SophoPublicUrl,
    SophoSelfServeTenantLimit,
    SophoPlatformAdministratorEmails,
    SophoDisabledConnectionTypes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Deserialize)]
pub enum SmtpTls {
    #[strum(serialize = "tls")]
    #[serde(rename = "tls")]
    Tls,
    #[strum(serialize = "starttls")]
    #[serde(rename = "starttls")]
    Starttls,
    #[strum(serialize = "none")]
    #[serde(rename = "none")]
    Unencrypted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OidcProviderVariable {
    DisplayName,
    IssuerUrl,
    ClientId,
    ClientSecret,
    RedirectUri,
}

impl OidcProviderVariable {
    pub fn name_for_provider(self, provider_name: &str) -> String {
        format!(
            "{OIDC_PROVIDER_VARIABLE_PREFIX}{}_{self}",
            provider_name.to_uppercase().replace('-', "_")
        )
    }
}
