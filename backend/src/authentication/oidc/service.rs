use crate::authentication::constants::UserStatus;
use crate::authentication::oidc::constants::{
    CallbackPurpose, EMAIL_SCOPE, GOOGLE_ISSUER_HOST, LOGIN_ATTEMPT_EXPIRY_MINUTES, PROFILE_SCOPE,
    ProviderIcon, SignInMethodChange,
};
use crate::authentication::oidc::dto::{
    CallbackOutcome, CallbackQuery, LoginSecrets, OidcProviderDto, SignInMethodsDto,
    SignInProviderDto, StartedLogin, VerifiedClaims,
};
use crate::authentication::oidc::error::{CallbackFailure, OidcLoginError, SignInMethodError};
use crate::authentication::oidc::repository;
use crate::authentication::repository as authentication_repository;
use crate::authentication::service as authentication_service;
use crate::common::time_utils;
use crate::common::{AppState, OidcProviderConfiguration};
use crate::email_delivery::dto::EmailMessage;
use crate::email_delivery::service as email_delivery_service;
use crate::email_template::dto::{SignInMethodAddedEmail, SignInMethodRemovedEmail};
use crate::email_template::service::{EmailTemplate, format_email_time};
use crate::entity::{identity, oidc_login_attempt, user};
use chrono::Utc;
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{DatabaseTransaction, TransactionTrait};
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
            icon: provider_icon(&provider.issuer_url),
        })
        .collect()
}

fn provider_icon(issuer_url: &str) -> ProviderIcon {
    let issuer_host = IssuerUrl::new(issuer_url.to_string())
        .ok()
        .and_then(|issuer| issuer.url().host_str().map(str::to_lowercase));
    match issuer_host.as_deref() {
        Some(GOOGLE_ISSUER_HOST) => ProviderIcon::Google,
        _ => ProviderIcon::Generic,
    }
}

pub async fn start_login(
    app_state: &AppState,
    provider_name: &str,
    linking_user_id: Option<Uuid>,
) -> Result<StartedLogin, OidcLoginError> {
    let provider = provider_configuration(app_state, provider_name)?;
    let (client, _) = build_oidc_client(provider).await?;
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
        linking_user_id,
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
) -> Result<CallbackOutcome, CallbackFailure> {
    let sign_in_failure = |error| CallbackFailure {
        purpose: CallbackPurpose::SignIn,
        error,
    };
    let Some(state) = query.state else {
        return Err(sign_in_failure(
            query
                .error
                .map(OidcLoginError::Provider)
                .unwrap_or(OidcLoginError::StateMismatch),
        ));
    };
    require_matching_state(cookie_state.as_deref(), &state).map_err(sign_in_failure)?;
    let attempt = consume_attempt(app_state, provider_name, &state)
        .await
        .map_err(sign_in_failure)?;
    let purpose = match attempt.linking_user_id {
        Some(_) => CallbackPurpose::Connect,
        None => CallbackPurpose::SignIn,
    };
    let failure = |error| CallbackFailure { purpose, error };
    if let Some(provider_error) = query.error {
        return Err(failure(OidcLoginError::Provider(provider_error)));
    }
    let code = query.code.ok_or_else(|| {
        failure(OidcLoginError::Provider(
            "the provider sent no authorization code".to_string(),
        ))
    })?;
    let provider = provider_configuration(app_state, provider_name).map_err(failure)?;
    let claims = verified_claims(provider, &attempt, code)
        .await
        .map_err(failure)?;

    match attempt.linking_user_id {
        Some(linking_user_id) => {
            connect_identity(app_state, linking_user_id, claims)
                .await
                .map_err(failure)?;
            Ok(CallbackOutcome::Connected {
                provider_name: provider_name.to_string(),
            })
        }
        None => {
            let user_id = resolve_user(app_state, claims).await.map_err(failure)?;
            authentication_service::start_session_for_user(app_state, cookies, user_id)
                .await
                .map_err(|error| failure(error.into()))?;
            Ok(CallbackOutcome::SignedIn)
        }
    }
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
    linking_user_id: Option<Uuid>,
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
            linking_user_id,
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
        let identity_owner = authentication_repository::get_user(
            &app_state.database_connection,
            existing_identity.user_id,
        )
        .await?
        .ok_or(OidcLoginError::AttemptNotFound)?;
        require_active(&identity_owner)?;
        return Ok(identity_owner.id);
    }

    let email = verified_email(&claims)?;
    if let Some(existing_user) =
        authentication_service::find_optional_user_by_email(app_state, &email).await?
    {
        require_active(&existing_user)?;
        if existing_user.email_verified_at.is_none() {
            return Err(OidcLoginError::AccountRequiresPasswordSignIn);
        }
        let has_identity_at_provider =
            repository::find_identities_for_user(&app_state.database_connection, existing_user.id)
                .await?
                .iter()
                .any(|identity| identity.provider_name == claims.provider_name);
        if has_identity_at_provider {
            return Err(OidcLoginError::SecondIdentityAtProvider);
        }
        add_identity(app_state, &existing_user, &claims, &email).await?;
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

pub async fn connect_identity(
    app_state: &AppState,
    linking_user_id: Uuid,
    claims: VerifiedClaims,
) -> Result<(), OidcLoginError> {
    let linking_user =
        authentication_repository::get_user(&app_state.database_connection, linking_user_id)
            .await?
            .ok_or(OidcLoginError::AttemptNotFound)?;
    require_active(&linking_user)?;
    let email = verified_email(&claims)?;
    if email != authentication_service::normalize_email(&linking_user.email) {
        return Err(OidcLoginError::EmailMismatch);
    }
    if let Some(existing_identity) = repository::find_identity(
        &app_state.database_connection,
        &claims.provider_name,
        &claims.subject,
    )
    .await?
    {
        return Err(match existing_identity.user_id == linking_user.id {
            true => OidcLoginError::AlreadyConnected,
            false => OidcLoginError::IdentityInUse,
        });
    }
    let has_identity_at_provider =
        repository::find_identities_for_user(&app_state.database_connection, linking_user.id)
            .await?
            .iter()
            .any(|identity| identity.provider_name == claims.provider_name);
    if has_identity_at_provider {
        return Err(OidcLoginError::AlreadyConnected);
    }
    add_identity(app_state, &linking_user, &claims, &email).await
}

pub async fn list_sign_in_methods(
    app_state: &AppState,
    user_id: Uuid,
) -> Result<SignInMethodsDto, SignInMethodError> {
    let account = authentication_repository::get_user(&app_state.database_connection, user_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(user_id.to_string()))?;
    let identities =
        repository::find_identities_for_user(&app_state.database_connection, user_id).await?;
    let has_password = account.password_hash.is_some();
    let can_disconnect = has_password || identities.len() > 1;

    let mut providers: Vec<SignInProviderDto> = app_state
        .config
        .oidc_providers
        .iter()
        .map(|provider| {
            let connected_identity = identities
                .iter()
                .find(|identity| identity.provider_name == provider.name);
            SignInProviderDto {
                name: provider.name.clone(),
                display_name: provider.display_name.clone(),
                connected: connected_identity.is_some(),
                email: connected_identity.map(|identity| identity.email.clone()),
                can_disconnect: connected_identity.is_some() && can_disconnect,
            }
        })
        .collect();
    providers.extend(
        identities
            .iter()
            .filter(|identity| {
                !app_state
                    .config
                    .oidc_providers
                    .iter()
                    .any(|provider| provider.name == identity.provider_name)
            })
            .map(|identity| SignInProviderDto {
                name: identity.provider_name.clone(),
                display_name: identity.provider_name.clone(),
                connected: true,
                email: Some(identity.email.clone()),
                can_disconnect,
            }),
    );
    Ok(SignInMethodsDto {
        has_password,
        providers,
    })
}

pub async fn disconnect_provider(
    app_state: &AppState,
    user_id: Uuid,
    provider_name: &str,
) -> Result<(), SignInMethodError> {
    let account = authentication_repository::get_user(&app_state.database_connection, user_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(user_id.to_string()))?;
    let identities =
        repository::find_identities_for_user(&app_state.database_connection, user_id).await?;
    let identity = identities
        .iter()
        .find(|identity| identity.provider_name == provider_name)
        .ok_or(SignInMethodError::NotConnected)?;
    if account.password_hash.is_none() && identities.len() <= 1 {
        return Err(SignInMethodError::LastSignInMethod);
    }
    let txn = app_state.database_connection.begin().await?;
    repository::delete_identity(&txn, identity.id).await?;
    queue_sign_in_method_notice_transaction(
        &txn,
        app_state,
        &account,
        provider_name,
        SignInMethodChange::Removed,
    )
    .await?;
    txn.commit().await?;
    Ok(())
}

async fn add_identity(
    app_state: &AppState,
    account: &user::Model,
    claims: &VerifiedClaims,
    email: &str,
) -> Result<(), OidcLoginError> {
    let txn = app_state.database_connection.begin().await?;
    repository::insert_identity(&txn, identity_for(account.id, claims, email)).await?;
    authentication_service::mark_email_verified_transaction(&txn, account.id).await?;
    queue_sign_in_method_notice_transaction(
        &txn,
        app_state,
        account,
        &claims.provider_name,
        SignInMethodChange::Added,
    )
    .await
    .map_err(|error| match error {
        SignInMethodError::EmailDelivery(error) => OidcLoginError::EmailDelivery(error),
        SignInMethodError::EmailTemplate(error) => OidcLoginError::EmailTemplate(error),
        SignInMethodError::Database(error) => OidcLoginError::Database(error),
        other => OidcLoginError::Internal(other.to_string()),
    })?;
    txn.commit().await?;
    Ok(())
}

async fn queue_sign_in_method_notice_transaction(
    txn: &DatabaseTransaction,
    app_state: &AppState,
    account: &user::Model,
    provider_name: &str,
    change: SignInMethodChange,
) -> Result<(), SignInMethodError> {
    if !email_delivery_service::is_email_enabled(app_state) {
        return Ok(());
    }
    let provider_display_name = provider_display_name(app_state, provider_name);
    let changed_at = format_email_time(Utc::now());
    let rendered_email = match change {
        SignInMethodChange::Added => SignInMethodAddedEmail {
            full_name: account.full_name.clone(),
            provider_display_name,
            changed_at,
        }
        .render()?,
        SignInMethodChange::Removed => SignInMethodRemovedEmail {
            full_name: account.full_name.clone(),
            provider_display_name,
            changed_at,
        }
        .render()?,
    };
    email_delivery_service::enqueue_email_transaction(
        txn,
        app_state,
        EmailMessage {
            kind: rendered_email.kind.to_string(),
            recipient: account.email.clone(),
            subject: rendered_email.subject,
            text_body: rendered_email.text_body,
            html_body: rendered_email.html_body,
            send_before: email_delivery_service::notice_send_before(),
            tenant_id: None,
            requested_by_user_id: Some(account.id),
        },
    )
    .await?;
    Ok(())
}

pub async fn connected_provider_display_names(
    app_state: &AppState,
    user_id: Uuid,
) -> Result<Vec<String>, sea_orm::DbErr> {
    Ok(
        repository::find_identities_for_user(&app_state.database_connection, user_id)
            .await?
            .iter()
            .map(|identity| provider_display_name(app_state, &identity.provider_name))
            .collect(),
    )
}

pub async fn provider_names_by_user(
    app_state: &AppState,
    user_ids: Vec<Uuid>,
) -> Result<std::collections::HashMap<Uuid, Vec<String>>, sea_orm::DbErr> {
    let mut provider_names: std::collections::HashMap<Uuid, Vec<String>> =
        std::collections::HashMap::new();
    for identity in
        repository::find_identities_for_users(&app_state.database_connection, user_ids).await?
    {
        provider_names
            .entry(identity.user_id)
            .or_default()
            .push(provider_display_name(app_state, &identity.provider_name));
    }
    Ok(provider_names)
}

fn provider_display_name(app_state: &AppState, provider_name: &str) -> String {
    app_state
        .config
        .oidc_providers
        .iter()
        .find(|provider| provider.name == provider_name)
        .map(|provider| provider.display_name.clone())
        .unwrap_or_else(|| provider_name.to_string())
}

fn require_active(account: &user::Model) -> Result<(), OidcLoginError> {
    if account.status == UserStatus::Deactivated.as_str() {
        return Err(OidcLoginError::AccountDeactivated);
    }
    Ok(())
}

fn verified_email(claims: &VerifiedClaims) -> Result<String, OidcLoginError> {
    let email = claims
        .email
        .as_deref()
        .map(authentication_service::normalize_email)
        .filter(|email| !email.is_empty())
        .ok_or(OidcLoginError::MissingEmail)?;
    if !claims.email_verified {
        return Err(OidcLoginError::EmailNotVerified);
    }
    Ok(email)
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

async fn build_oidc_client(
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
    let (client, http_client) = build_oidc_client(provider).await?;
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

#[cfg(test)]
mod tests {
    use super::provider_icon;
    use crate::authentication::oidc::constants::ProviderIcon;

    #[test]
    fn the_icon_comes_from_the_issuer_host() {
        assert_eq!(
            provider_icon("https://accounts.google.com"),
            ProviderIcon::Google
        );
        assert_eq!(
            provider_icon("https://accounts.google.com/"),
            ProviderIcon::Google
        );
        assert_eq!(
            provider_icon("https://acme.okta.com"),
            ProviderIcon::Generic
        );
        assert_eq!(
            provider_icon("https://accounts.google.com.evil.example"),
            ProviderIcon::Generic
        );
    }
}
