use strum::{AsRefStr, Display, EnumString};

pub const API_PATH_PREFIX: &str = "/api/";
pub const OIDC_PROVIDER_VARIABLE_PREFIX: &str = "SOPHO_OIDC_";
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
