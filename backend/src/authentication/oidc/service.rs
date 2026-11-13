use crate::authentication::constants::UserStatus;
use crate::authentication::oidc::constants::{
    AFTER_SIGN_IN_PATH, CONNECT_RESULT_PARAMETER, CONNECTED_PROVIDER_PARAMETER, CallbackPurpose,
    ConnectResult, EMAIL_SCOPE, GOOGLE_ISSUER_HOST, LOGIN_ATTEMPT_EXPIRY_MINUTES, OIDC_COOKIE_PATH,
    OIDC_STATE_COOKIE_NAME, ONLY_SIGN_IN_METHOD_REASON, PROFILE_PATH, PROFILE_SCOPE, ProviderIcon,
    SIGN_IN_ERROR_PARAMETER, SIGN_IN_PATH, SignInErrorCode, SignInMethodChange,
};
use crate::authentication::oidc::dto::{
    CallbackOutcome, CallbackQuery, LoginSecrets, MySignInMethodsDto, OidcProviderDto,
    SignInProviderDto, StartedLogin, VerifiedClaims,
};
use crate::authentication::oidc::error::{CallbackFailure, OidcLoginError, SignInMethodError};
use crate::authentication::oidc::repository;
use crate::authentication::repository as authentication_repository;
use crate::authentication::service as authentication_service;
use crate::authorization::constants::PlatformRole;
use crate::common::time_utils;
use crate::common::{AppState, OidcProviderConfiguration};
use crate::email_delivery::dto::EmailMessage;
use crate::email_delivery::service as email_delivery_service;
use crate::email_template::dto::{SignInMethodAddedEmail, SignInMethodRemovedEmail};
use crate::email_template::service::{EmailTemplate, format_email_time};
use crate::entity::{federated_identity, oidc_login_attempt, user};
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
use tower_cookies::cookie::{Cookie, SameSite};
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

pub async fn start_sign_in(app_state: &AppState, cookies: &Cookies, provider_name: &str) -> String {
    match start_login(app_state, provider_name, None).await {
        Ok(started) => {
            cookies.add(state_cookie(app_state, started.state));
            started.authorization_url
        }
        Err(error) => {
            tracing::error!("Sign-in with {provider_name} could not start: {error}");
            sign_in_failure_path(&error)
        }
    }
}

pub async fn start_connect(
    app_state: &AppState,
    cookies: &Cookies,
    provider_name: &str,
    linking_user_id: Uuid,
) -> String {
    match start_login(app_state, provider_name, Some(linking_user_id)).await {
        Ok(started) => {
            cookies.add(state_cookie(app_state, started.state));
            started.authorization_url
        }
        Err(error) => {
            tracing::error!("Connecting {provider_name} could not start: {error}");
            connect_failure_path(error.connect_result())
        }
    }
}

pub async fn complete_callback(
    app_state: &AppState,
    cookies: &Cookies,
    provider_name: &str,
    query: CallbackQuery,
) -> String {
    let cookie_state = cookies
        .get(OIDC_STATE_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());
    cookies.remove(state_cookie_removal());

    match complete_login(app_state, cookies, provider_name, query, cookie_state).await {
        Ok(CallbackOutcome::SignedIn) => AFTER_SIGN_IN_PATH.to_string(),
        Ok(CallbackOutcome::Connected { provider_name }) => format!(
            "{PROFILE_PATH}?{CONNECT_RESULT_PARAMETER}={}&{CONNECTED_PROVIDER_PARAMETER}={provider_name}",
            ConnectResult::Connected.as_str()
        ),
        Err(failure) => {
            tracing::error!(
                "Single sign-on with {provider_name} failed: {}",
                failure.error
            );
            match failure.purpose {
                CallbackPurpose::SignIn => sign_in_failure_path(&failure.error),
                CallbackPurpose::Connect => connect_failure_path(failure.error.connect_result()),
            }
        }
    }
}

pub fn too_many_attempts_sign_in_path() -> String {
    format!(
        "{SIGN_IN_PATH}?{SIGN_IN_ERROR_PARAMETER}={}",
        SignInErrorCode::TooManyAttempts.as_str()
    )
}

fn sign_in_failure_path(error: &OidcLoginError) -> String {
    format!(
        "{SIGN_IN_PATH}?{SIGN_IN_ERROR_PARAMETER}={}",
        error.sign_in_error_code().as_str()
    )
}

fn connect_failure_path(connect_result: ConnectResult) -> String {
    format!(
        "{PROFILE_PATH}?{CONNECT_RESULT_PARAMETER}={}",
        connect_result.as_str()
    )
}

fn state_cookie(app_state: &AppState, state: String) -> Cookie<'static> {
    Cookie::build((OIDC_STATE_COOKIE_NAME, state))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path(OIDC_COOKIE_PATH)
        .secure(app_state.config.cookie_secure)
        .max_age(time::Duration::minutes(LOGIN_ATTEMPT_EXPIRY_MINUTES))
        .build()
}

fn state_cookie_removal() -> Cookie<'static> {
    Cookie::build((OIDC_STATE_COOKIE_NAME, ""))
        .path(OIDC_COOKIE_PATH)
        .build()
}

async fn start_login(
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

async fn complete_login(
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

    let provider_verified_email = verified_email(&claims).ok();
    match attempt.linking_user_id {
        Some(linking_user_id) => {
            connect_federated_identity(app_state, linking_user_id, claims)
                .await
                .map_err(failure)?;
            promote_listed_platform_administrator(
                app_state,
                linking_user_id,
                provider_name,
                provider_verified_email.as_deref(),
            )
            .await
            .map_err(failure)?;
            Ok(CallbackOutcome::Connected {
                provider_name: provider_name.to_string(),
            })
        }
        None => {
            let user_id = resolve_user(app_state, claims).await.map_err(failure)?;
            promote_listed_platform_administrator(
                app_state,
                user_id,
                provider_name,
                provider_verified_email.as_deref(),
            )
            .await
            .map_err(failure)?;
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
    if let Some(existing_federated_identity) = repository::find_federated_identity(
        &app_state.database_connection,
        &claims.provider_name,
        &claims.subject,
    )
    .await?
    {
        let federated_identity_owner = authentication_repository::get_user(
            &app_state.database_connection,
            existing_federated_identity.user_id,
        )
        .await?
        .ok_or(OidcLoginError::AttemptNotFound)?;
        require_active(&federated_identity_owner)?;
        tracing::info!(
            "Single sign-on with {} matched the federated identity of user {}",
            claims.provider_name,
            federated_identity_owner.id
        );
        return Ok(federated_identity_owner.id);
    }

    let email = verified_email(&claims)?;
    if let Some(existing_user) =
        authentication_service::find_optional_user_by_email(app_state, &email).await?
    {
        require_active(&existing_user)?;
        if existing_user.email_verified_at.is_none() {
            return Err(
                match authentication_service::is_password_sign_in_available(app_state) {
                    true => OidcLoginError::AccountRequiresPasswordSignIn,
                    false => OidcLoginError::AccountCannotSignIn,
                },
            );
        }
        let has_federated_identity_at_provider = repository::find_federated_identities_for_user(
            &app_state.database_connection,
            existing_user.id,
        )
        .await?
        .iter()
        .any(|federated_identity| federated_identity.provider_name == claims.provider_name);
        if has_federated_identity_at_provider {
            return Err(OidcLoginError::SecondFederatedIdentityAtProvider);
        }
        add_federated_identity(app_state, &existing_user, &claims, &email).await?;
        tracing::info!(
            "Single sign-on with {} was linked to existing user {} by verified email",
            claims.provider_name,
            existing_user.id
        );
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
    repository::insert_federated_identity(&txn, federated_identity_for(user.id, &claims, &email))
        .await?;
    txn.commit().await?;
    tracing::info!(
        "Single sign-on with {} created new user {}",
        claims.provider_name,
        user.id
    );
    Ok(user.id)
}

async fn promote_listed_platform_administrator(
    app_state: &AppState,
    user_id: Uuid,
    provider_name: &str,
    provider_verified_email: Option<&str>,
) -> Result<(), OidcLoginError> {
    let Some(provider_verified_email) = provider_verified_email else {
        return Ok(());
    };
    let is_listed = app_state
        .config
        .platform_administrator_emails
        .iter()
        .any(|listed_email| listed_email == provider_verified_email);
    if !is_listed {
        return Ok(());
    }
    let account = authentication_repository::get_user(&app_state.database_connection, user_id)
        .await?
        .ok_or(OidcLoginError::AttemptNotFound)?;
    require_active(&account)?;
    if account.platform_role == PlatformRole::Administrator.as_str() {
        return Ok(());
    }
    authentication_service::set_platform_role(app_state, account.id, PlatformRole::Administrator)
        .await?;
    tracing::info!(
        "User {} became a platform administrator by signing in with {provider_name}: their verified email is in the platform administrator list",
        account.id
    );
    Ok(())
}

fn has_another_way_to_sign_in(
    app_state: &AppState,
    account: &user::Model,
    federated_identities: &[federated_identity::Model],
    federated_identity_id: Uuid,
) -> bool {
    if authentication_service::is_password_sign_in_available(app_state) {
        return account.password_hash.is_some() || federated_identities.len() > 1;
    }
    federated_identities.iter().any(|other_federated_identity| {
        other_federated_identity.id != federated_identity_id
            && app_state
                .config
                .oidc_providers
                .iter()
                .any(|provider| provider.name == other_federated_identity.provider_name)
    })
}

pub async fn connect_federated_identity(
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
    if let Some(existing_federated_identity) = repository::find_federated_identity(
        &app_state.database_connection,
        &claims.provider_name,
        &claims.subject,
    )
    .await?
    {
        return Err(
            match existing_federated_identity.user_id == linking_user.id {
                true => OidcLoginError::AlreadyConnected,
                false => OidcLoginError::FederatedIdentityInUse,
            },
        );
    }
    let has_federated_identity_at_provider = repository::find_federated_identities_for_user(
        &app_state.database_connection,
        linking_user.id,
    )
    .await?
    .iter()
    .any(|federated_identity| federated_identity.provider_name == claims.provider_name);
    if has_federated_identity_at_provider {
        return Err(OidcLoginError::AlreadyConnected);
    }
    add_federated_identity(app_state, &linking_user, &claims, &email).await
}

pub async fn list_my_sign_in_methods(
    app_state: &AppState,
    user_id: Uuid,
) -> Result<MySignInMethodsDto, SignInMethodError> {
    let account = authentication_repository::get_user(&app_state.database_connection, user_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(user_id.to_string()))?;
    let federated_identities =
        repository::find_federated_identities_for_user(&app_state.database_connection, user_id)
            .await?;
    let has_password = account.password_hash.is_some();
    let can_disconnect_federated_identity = |federated_identity: &federated_identity::Model| {
        has_another_way_to_sign_in(
            app_state,
            &account,
            &federated_identities,
            federated_identity.id,
        )
    };
    let disconnect_blocked_reason =
        |can_disconnect: bool| (!can_disconnect).then(|| ONLY_SIGN_IN_METHOD_REASON.to_string());

    let mut providers: Vec<SignInProviderDto> = app_state
        .config
        .oidc_providers
        .iter()
        .map(|provider| {
            let connected_federated_identity = federated_identities
                .iter()
                .find(|federated_identity| federated_identity.provider_name == provider.name);
            let can_disconnect =
                connected_federated_identity.is_some_and(can_disconnect_federated_identity);
            SignInProviderDto {
                name: provider.name.clone(),
                display_name: provider.display_name.clone(),
                connected: connected_federated_identity.is_some(),
                email: connected_federated_identity
                    .map(|federated_identity| federated_identity.email.clone()),
                can_disconnect,
                disconnect_blocked_reason: connected_federated_identity
                    .and_then(|_| disconnect_blocked_reason(can_disconnect)),
            }
        })
        .collect();
    providers.extend(
        federated_identities
            .iter()
            .filter(|federated_identity| {
                !app_state
                    .config
                    .oidc_providers
                    .iter()
                    .any(|provider| provider.name == federated_identity.provider_name)
            })
            .map(|federated_identity| {
                let can_disconnect = can_disconnect_federated_identity(federated_identity);
                SignInProviderDto {
                    name: federated_identity.provider_name.clone(),
                    display_name: federated_identity.provider_name.clone(),
                    connected: true,
                    email: Some(federated_identity.email.clone()),
                    can_disconnect,
                    disconnect_blocked_reason: disconnect_blocked_reason(can_disconnect),
                }
            }),
    );
    Ok(MySignInMethodsDto {
        has_password,
        password_available: authentication_service::is_password_sign_in_available(app_state),
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
    let federated_identities =
        repository::find_federated_identities_for_user(&app_state.database_connection, user_id)
            .await?;
    let federated_identity = federated_identities
        .iter()
        .find(|federated_identity| federated_identity.provider_name == provider_name)
        .ok_or(SignInMethodError::NotConnected)?;
    if !has_another_way_to_sign_in(
        app_state,
        &account,
        &federated_identities,
        federated_identity.id,
    ) {
        return Err(SignInMethodError::LastSignInMethod);
    }
    let txn = app_state.database_connection.begin().await?;
    repository::delete_federated_identity(&txn, federated_identity.id).await?;
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

async fn add_federated_identity(
    app_state: &AppState,
    account: &user::Model,
    claims: &VerifiedClaims,
    email: &str,
) -> Result<(), OidcLoginError> {
    let txn = app_state.database_connection.begin().await?;
    repository::insert_federated_identity(&txn, federated_identity_for(account.id, claims, email))
        .await?;
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
        tracing::info!(
            "Email is not configured, so user {} gets no notice about the {} sign-in method change",
            account.id,
            provider_name
        );
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
        repository::find_federated_identities_for_user(&app_state.database_connection, user_id)
            .await?
            .iter()
            .map(|federated_identity| {
                provider_display_name(app_state, &federated_identity.provider_name)
            })
            .collect(),
    )
}

pub async fn provider_names_by_user(
    app_state: &AppState,
    user_ids: Vec<Uuid>,
) -> Result<std::collections::HashMap<Uuid, Vec<String>>, sea_orm::DbErr> {
    let mut provider_names: std::collections::HashMap<Uuid, Vec<String>> =
        std::collections::HashMap::new();
    for federated_identity in
        repository::find_federated_identities_for_users(&app_state.database_connection, user_ids)
            .await?
    {
        provider_names
            .entry(federated_identity.user_id)
            .or_default()
            .push(provider_display_name(
                app_state,
                &federated_identity.provider_name,
            ));
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

fn federated_identity_for(
    user_id: Uuid,
    claims: &VerifiedClaims,
    email: &str,
) -> federated_identity::Model {
    federated_identity::Model {
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
