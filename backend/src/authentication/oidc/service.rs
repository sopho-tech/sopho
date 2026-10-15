use crate::authentication::oidc::constants::{
    EMAIL_SCOPE, LOGIN_ATTEMPT_EXPIRY_MINUTES, PROFILE_SCOPE,
};
use crate::authentication::oidc::dto::{
    CallbackQuery, LoginSecrets, OidcProviderDto, StartedLogin, VerifiedClaims,
};
use crate::authentication::oidc::error::OidcLoginError;
use crate::authentication::oidc::repository;
use crate::authentication::service as authentication_service;
use crate::common::time_utils;
use crate::common::{AppState, OidcProviderConfiguration};
use crate::entity::{identity, oidc_login_attempt};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
};
use sea_orm::TransactionTrait;
use sea_orm::prelude::DateTimeWithTimeZone;
use tower_cookies::Cookies;
use uuid::Uuid;

type DiscoveredClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub fn list_providers(app_state: &AppState) -> Vec<OidcProviderDto> {
    app_state
        .config
        .oidc_providers
        .iter()
        .map(|provider| OidcProviderDto {
            name: provider.name.clone(),
            display_name: provider.display_name.clone(),
        })
        .collect()
}

pub async fn start_login(
    app_state: &AppState,
    provider_name: &str,
) -> Result<StartedLogin, OidcLoginError> {
    let provider = provider_configuration(app_state, provider_name)?;
    let (client, _) = discover_client(provider).await?;
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorization_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new(EMAIL_SCOPE.to_string()))
        .add_scope(Scope::new(PROFILE_SCOPE.to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let state = csrf_token.secret().clone();
    record_login_attempt(
        app_state,
        &provider.name,
        LoginSecrets {
            state: state.clone(),
            nonce: nonce.secret().clone(),
            code_verifier: pkce_verifier.secret().clone(),
        },
    )
    .await?;

    Ok(StartedLogin {
        authorization_url: authorization_url.to_string(),
        state,
    })
}

pub async fn complete_login(
    app_state: &AppState,
    cookies: &Cookies,
    provider_name: &str,
    query: CallbackQuery,
    cookie_state: Option<String>,
) -> Result<(), OidcLoginError> {
    if let Some(provider_error) = query.error {
        return Err(OidcLoginError::Provider(provider_error));
    }
    let state = query.state.ok_or(OidcLoginError::StateMismatch)?;
    require_matching_state(cookie_state.as_deref(), &state)?;
    let attempt = consume_attempt(app_state, provider_name, &state).await?;
    let code = query.code.ok_or_else(|| {
        OidcLoginError::Provider("the provider sent no authorization code".to_string())
    })?;

    let provider = provider_configuration(app_state, provider_name)?;
    let claims = verified_claims(provider, &attempt, code).await?;
    let user_id = resolve_user(app_state, claims).await?;
    authentication_service::start_session_for_user(app_state, cookies, user_id).await?;
    Ok(())
}

pub fn require_matching_state(
    cookie_state: Option<&str>,
    query_state: &str,
) -> Result<(), OidcLoginError> {
    match cookie_state {
        Some(cookie_state) if equal_in_constant_time(cookie_state, query_state) => Ok(()),
        _ => Err(OidcLoginError::StateMismatch),
    }
}

pub async fn record_login_attempt(
    app_state: &AppState,
    provider_name: &str,
    secrets: LoginSecrets,
) -> Result<(), OidcLoginError> {
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    repository::delete_expired_login_attempts(&app_state.database_connection, now).await?;
    repository::insert_login_attempt(
        &app_state.database_connection,
        oidc_login_attempt::Model {
            id: Uuid::new_v4(),
            provider_name: provider_name.to_string(),
            state: secrets.state,
            nonce: secrets.nonce,
            code_verifier: secrets.code_verifier,
            expires_at: time_utils::now_plus_minutes_into(LOGIN_ATTEMPT_EXPIRY_MINUTES),
            created_at: now,
            updated_at: now,
        },
    )
    .await?;
    Ok(())
}

pub async fn consume_attempt(
    app_state: &AppState,
    provider_name: &str,
    state: &str,
) -> Result<oidc_login_attempt::Model, OidcLoginError> {
    let attempt = repository::find_login_attempt_by_state(&app_state.database_connection, state)
        .await?
        .ok_or(OidcLoginError::AttemptNotFound)?;
    let consumed =
        repository::delete_login_attempt(&app_state.database_connection, attempt.id).await?;
    if consumed == 0 || attempt.provider_name != provider_name {
        return Err(OidcLoginError::AttemptNotFound);
    }
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    if attempt.expires_at <= now {
        return Err(OidcLoginError::AttemptExpired);
    }
    Ok(attempt)
}

pub async fn resolve_user(
    app_state: &AppState,
    claims: VerifiedClaims,
) -> Result<Uuid, OidcLoginError> {
    if let Some(existing_identity) = repository::find_identity(
        &app_state.database_connection,
        &claims.provider_name,
        &claims.subject,
    )
    .await?
    {
        return Ok(existing_identity.user_id);
    }

    let email = claims
        .email
        .as_deref()
        .map(authentication_service::normalize_email)
        .filter(|email| !email.is_empty())
        .ok_or(OidcLoginError::MissingEmail)?;
    if !claims.email_verified {
        return Err(OidcLoginError::EmailNotVerified);
    }

    if let Some(existing_user) =
        authentication_service::find_optional_user_by_email(app_state, &email).await?
    {
        let verified_by_a_provider =
            repository::count_identities_for_user(&app_state.database_connection, existing_user.id)
                .await?
                > 0;
        if !verified_by_a_provider {
            return Err(OidcLoginError::AccountRequiresPasswordSignIn);
        }
        repository::insert_identity(
            &app_state.database_connection,
            identity_for(existing_user.id, &claims, &email),
        )
        .await?;
        return Ok(existing_user.id);
    }

    let full_name = claims
        .full_name
        .as_deref()
        .map(str::trim)
        .filter(|full_name| !full_name.is_empty())
        .ok_or(OidcLoginError::MissingProfile)?;
    let txn = app_state.database_connection.begin().await?;
    let user = authentication_service::create_sso_user_transaction(&txn, &email, full_name).await?;
    repository::insert_identity(&txn, identity_for(user.id, &claims, &email)).await?;
    txn.commit().await?;
    Ok(user.id)
}

fn identity_for(user_id: Uuid, claims: &VerifiedClaims, email: &str) -> identity::Model {
    identity::Model {
        id: Uuid::new_v4(),
        user_id,
        provider_name: claims.provider_name.clone(),
        subject: claims.subject.clone(),
        email: email.to_string(),
        email_verified: claims.email_verified,
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    }
}

fn provider_configuration<'a>(
    app_state: &'a AppState,
    provider_name: &str,
) -> Result<&'a OidcProviderConfiguration, OidcLoginError> {
    app_state
        .config
        .oidc_providers
        .iter()
        .find(|provider| provider.name == provider_name)
        .ok_or_else(|| OidcLoginError::UnknownProvider(provider_name.to_string()))
}

async fn discover_client(
    provider: &OidcProviderConfiguration,
) -> Result<(DiscoveredClient, openidconnect::reqwest::Client), OidcLoginError> {
    let http_client = openidconnect::reqwest::ClientBuilder::new()
        .redirect(openidconnect::reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| OidcLoginError::Internal(format!("HTTP client: {error}")))?;
    let issuer_url = IssuerUrl::new(provider.issuer_url.clone())
        .map_err(|error| OidcLoginError::Internal(format!("issuer URL: {error}")))?;
    let redirect_url = RedirectUrl::new(provider.redirect_uri.clone())
        .map_err(|error| OidcLoginError::Internal(format!("redirect URI: {error}")))?;
    let metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .map_err(|error| OidcLoginError::Provider(format!("discovery failed: {error}")))?;
    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(provider.client_id.clone()),
        Some(ClientSecret::new(provider.client_secret.clone())),
    )
    .set_redirect_uri(redirect_url);
    Ok((client, http_client))
}

async fn verified_claims(
    provider: &OidcProviderConfiguration,
    attempt: &oidc_login_attempt::Model,
    code: String,
) -> Result<VerifiedClaims, OidcLoginError> {
    let (client, http_client) = discover_client(provider).await?;
    let token_response = client
        .exchange_code(AuthorizationCode::new(code))
        .map_err(|error| OidcLoginError::Provider(format!("token endpoint: {error}")))?
        .set_pkce_verifier(PkceCodeVerifier::new(attempt.code_verifier.clone()))
        .request_async(&http_client)
        .await
        .map_err(|error| OidcLoginError::Provider(format!("code exchange failed: {error}")))?;
    let id_token = token_response
        .id_token()
        .ok_or_else(|| OidcLoginError::Provider("the provider sent no ID token".to_string()))?;
    let claims = id_token
        .claims(
            &client.id_token_verifier(),
            &Nonce::new(attempt.nonce.clone()),
        )
        .map_err(|error| OidcLoginError::Provider(format!("ID token rejected: {error}")))?;

    Ok(VerifiedClaims {
        provider_name: provider.name.clone(),
        subject: claims.subject().as_str().to_string(),
        email: claims.email().map(|email| email.as_str().to_string()),
        email_verified: claims.email_verified() == Some(true),
        full_name: claims
            .name()
            .and_then(|name| name.get(None))
            .map(|name| name.as_str().to_string()),
    })
}

fn equal_in_constant_time(first: &str, second: &str) -> bool {
    first.len() == second.len()
        && first
            .bytes()
            .zip(second.bytes())
            .fold(0u8, |difference, (left, right)| difference | (left ^ right))
            == 0
}
