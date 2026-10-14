use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct OidcProviderDto {
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedClaims {
    pub provider_name: String,
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub full_name: Option<String>,
}

#[derive(Debug)]
pub struct LoginSecrets {
    pub state: String,
    pub nonce: String,
    pub code_verifier: String,
}

#[derive(Debug)]
pub struct StartedLogin {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}
