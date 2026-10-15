use crate::support::constants::{
    AFTER_SIGN_IN_LOCATION, API_PREFIX, MOCK_IDENTITY_CLAIMS_FIELD, MOCK_IDENTITY_CLIENT_ID,
    MOCK_IDENTITY_CLIENT_SECRET, MOCK_IDENTITY_CONFIGURATION, MOCK_IDENTITY_CONTAINER_NAME,
    MOCK_IDENTITY_DISCOVERY_PATH, MOCK_IDENTITY_IMAGE_NAME, MOCK_IDENTITY_IMAGE_TAG,
    MOCK_IDENTITY_PORT, MOCK_IDENTITY_READY_POLL_MILLISECONDS, MOCK_IDENTITY_READY_TIMEOUT_SECONDS,
    MOCK_IDENTITY_STARTUP_LOCK_FILE_NAME, MOCK_IDENTITY_SUBJECT_FIELD,
    OTHER_SINGLE_SIGN_ON_PROVIDER, OTHER_SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME,
    OTHER_SINGLE_SIGN_ON_PROVIDER_ISSUER, SIGN_IN_ERROR_LOCATION_PREFIX, SINGLE_SIGN_ON_PROVIDER,
    SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME, SINGLE_SIGN_ON_PROVIDER_ISSUER,
};
use crate::support::test_app::UserSession;
use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use reqwest::{Client, Response, StatusCode};
use serde_json::Value;
use sopho::OidcProviderConfiguration;
use std::fs::File;
use std::time::{Duration, Instant};
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt, ReuseDirective};
use tokio::sync::OnceCell;

pub struct MockIdentityServer {
    base_url: String,
}

impl MockIdentityServer {
    fn issuer_url(&self, issuer: &str) -> String {
        format!("{}/{issuer}", self.base_url)
    }
}

static SHARED_MOCK_IDENTITY_SERVER: OnceCell<MockIdentityServer> = OnceCell::const_new();

async fn shared_mock_identity_server() -> &'static MockIdentityServer {
    SHARED_MOCK_IDENTITY_SERVER
        .get_or_init(start_or_reuse_mock_identity_server)
        .await
}

pub async fn mock_identity_providers(app_base_url: &str) -> Vec<OidcProviderConfiguration> {
    let server = shared_mock_identity_server().await;
    [
        (
            SINGLE_SIGN_ON_PROVIDER,
            SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME,
            SINGLE_SIGN_ON_PROVIDER_ISSUER,
        ),
        (
            OTHER_SINGLE_SIGN_ON_PROVIDER,
            OTHER_SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME,
            OTHER_SINGLE_SIGN_ON_PROVIDER_ISSUER,
        ),
    ]
    .into_iter()
    .map(|(name, display_name, issuer)| OidcProviderConfiguration {
        name: name.to_string(),
        display_name: display_name.to_string(),
        issuer_url: server.issuer_url(issuer),
        client_id: MOCK_IDENTITY_CLIENT_ID.to_string(),
        client_secret: MOCK_IDENTITY_CLIENT_SECRET.to_string(),
        redirect_uri: format!("{app_base_url}{API_PREFIX}/auth/oidc/{name}/callback"),
    })
    .collect()
}

async fn start_or_reuse_mock_identity_server() -> MockIdentityServer {
    let startup_lock = tokio::task::spawn_blocking(|| {
        let lock_file =
            File::create(std::env::temp_dir().join(MOCK_IDENTITY_STARTUP_LOCK_FILE_NAME))
                .expect("create the identity provider startup lock file");
        lock_file
            .lock()
            .expect("lock the identity provider startup lock file");
        lock_file
    })
    .await
    .expect("acquire the identity provider startup lock");
    let container = GenericImage::new(MOCK_IDENTITY_IMAGE_NAME, MOCK_IDENTITY_IMAGE_TAG)
        .with_exposed_port(MOCK_IDENTITY_PORT.tcp())
        .with_env_var("JSON_CONFIG", MOCK_IDENTITY_CONFIGURATION)
        .with_container_name(MOCK_IDENTITY_CONTAINER_NAME)
        .with_reuse(ReuseDirective::Always)
        .start()
        .await
        .unwrap_or_else(|error| {
            panic!(
                "single sign-on tests need a running Docker daemon (set DOCKER_HOST for OrbStack or colima): {error}"
            )
        });
    let server = MockIdentityServer {
        base_url: format!(
            "http://{}:{}",
            container
                .get_host()
                .await
                .expect("read the identity provider host"),
            container
                .get_host_port_ipv4(MOCK_IDENTITY_PORT)
                .await
                .expect("read the identity provider port")
        ),
    };
    wait_until_ready(&server).await;
    drop(startup_lock);
    server
}

async fn wait_until_ready(server: &MockIdentityServer) {
    let discovery_url = format!(
        "{}/{MOCK_IDENTITY_DISCOVERY_PATH}",
        server.issuer_url(SINGLE_SIGN_ON_PROVIDER_ISSUER)
    );
    let deadline = Instant::now() + Duration::from_secs(MOCK_IDENTITY_READY_TIMEOUT_SECONDS);
    loop {
        let ready = matches!(
            reqwest::get(&discovery_url).await,
            Ok(response) if response.status() == StatusCode::OK
        );
        if ready {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the identity provider at {discovery_url} did not become ready"
        );
        tokio::time::sleep(Duration::from_millis(MOCK_IDENTITY_READY_POLL_MILLISECONDS)).await;
    }
}

pub struct StartedSignIn {
    pub authorization_url: String,
    pub state: String,
}

pub struct SignInOutcome {
    pub location: String,
}

impl SignInOutcome {
    pub fn is_signed_in(&self) -> bool {
        self.location == AFTER_SIGN_IN_LOCATION
    }

    pub fn error_code(&self) -> Option<&str> {
        self.location.strip_prefix(SIGN_IN_ERROR_LOCATION_PREFIX)
    }
}

fn redirect_location(response: &Response) -> String {
    assert!(
        response.status().is_redirection(),
        "expected a redirect from {}, got {}",
        response.url(),
        response.status()
    );
    response
        .headers()
        .get(LOCATION)
        .expect("the redirect has a location")
        .to_str()
        .expect("the location is text")
        .to_string()
}

pub async fn start_sign_in(session: &UserSession, provider_name: &str) -> StartedSignIn {
    let response = session
        .client_without_redirects()
        .get(session.api_url(&format!("/auth/oidc/{provider_name}/start")))
        .send()
        .await
        .expect("start the single sign-on");
    let authorization_url = redirect_location(&response);
    let state = reqwest::Url::parse(&authorization_url)
        .expect("the authorization URL is a URL")
        .query_pairs()
        .find(|(name, _)| name == "state")
        .map(|(_, value)| value.into_owned())
        .expect("the authorization URL carries a state");
    StartedSignIn {
        authorization_url,
        state,
    }
}

pub async fn sign_in_at_provider(started: &StartedSignIn, subject: &str, claims: Value) -> String {
    let response = Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("build the identity provider client")
        .post(&started.authorization_url)
        .form(&[
            (MOCK_IDENTITY_SUBJECT_FIELD, subject.to_string()),
            (MOCK_IDENTITY_CLAIMS_FIELD, claims.to_string()),
        ])
        .send()
        .await
        .expect("sign in at the identity provider");
    redirect_location(&response)
}

pub async fn complete_sign_in(session: &UserSession, callback_url: &str) -> SignInOutcome {
    let response = session
        .client_without_redirects()
        .get(callback_url)
        .send()
        .await
        .expect("return to the application");
    SignInOutcome {
        location: redirect_location(&response),
    }
}

pub async fn sign_in_with_provider(
    session: &UserSession,
    provider_name: &str,
    subject: &str,
    claims: Value,
) -> SignInOutcome {
    let started = start_sign_in(session, provider_name).await;
    let callback_url = sign_in_at_provider(&started, subject, claims).await;
    complete_sign_in(session, &callback_url).await
}
