use crate::support::constants::{
    ADMIN_EMAIL, ADMIN_FULL_NAME, ADMIN_PASSWORD, ADMIN_USERNAME, AI_PROVIDER_API_KEY, API_PREFIX,
    ENCRYPTION_KEY, ENVIRONMENT, FAKE_LLM_FIXTURES_RELATIVE_PATH, FRONTEND_DIRECTORY_FOR_API_TESTS,
    NO_DEFAULT_TENANT_ID, POSTGRES_SAMPLE_CONNECTION_NAME, SAMPLE_DATABASE_SCHEMA,
    SQLITE_DEMO_CONNECTION_NAME, TENANT_PATH_SEGMENT, TENANT_SCOPED_ROUTE_GROUPS,
};
use crate::support::data_sources::{
    SqliteDemoDatabase, copy_sqlite_demo_database, create_postgres_sample_database, manifest_path,
};
use crate::support::identity_provider::mock_identity_providers;
use crate::support::mail_server::{
    SilentMailServer, shared_mail_server, shared_refusing_mail_server,
    unreachable_email_configuration,
};
use crate::support::metadata_database::MetadataDatabase;
use crate::support::postgres_server::{PostgresDatabase, shared_postgres_server};
use fake_llm::FakeLlmServer;
use reqwest::cookie::Jar;
use reqwest::redirect::Policy;
use reqwest::{Client, Method, StatusCode};
use sea_orm::ConnectionTrait;
use sea_orm::sea_query::{Alias, Query};
use serde_json::{Value, json};
use sopho::{
    AppState, Configurations, EmailConfiguration, FeatureFlag, OidcProviderConfiguration,
    RateLimitSettings,
};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct JsonResponse {
    pub status: StatusCode,
    pub body: Value,
}

pub struct UserSession {
    pub client: Client,
    cookie_jar: Arc<Jar>,
    base_url: String,
    tenant_id: Option<Uuid>,
}

impl UserSession {
    pub fn new(base_url: &str, tenant_id: Option<Uuid>) -> Self {
        let cookie_jar = Arc::new(Jar::default());
        Self {
            client: Client::builder()
                .cookie_provider(cookie_jar.clone())
                .build()
                .expect("build the HTTP client"),
            cookie_jar,
            base_url: base_url.to_string(),
            tenant_id,
        }
    }

    pub fn in_tenant(&self, tenant_id: Uuid) -> Self {
        Self {
            client: self.client.clone(),
            cookie_jar: self.cookie_jar.clone(),
            base_url: self.base_url.clone(),
            tenant_id: Some(tenant_id),
        }
    }

    pub fn client_without_redirects(&self) -> Client {
        Client::builder()
            .cookie_provider(self.cookie_jar.clone())
            .redirect(Policy::none())
            .build()
            .expect("build the HTTP client")
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_url(&self, path: &str) -> String {
        format!("{}{API_PREFIX}{}", self.base_url, self.routed_path(path))
    }

    fn routed_path(&self, path: &str) -> String {
        let route_group = path
            .trim_start_matches('/')
            .split(['/', '?'])
            .next()
            .unwrap_or_default();
        if !TENANT_SCOPED_ROUTE_GROUPS.contains(&route_group) {
            return path.to_string();
        }
        let tenant_id = self
            .tenant_id
            .unwrap_or_else(|| panic!("{path} is tenant-scoped but this session has none"));
        format!("/{TENANT_PATH_SEGMENT}/{tenant_id}{path}")
    }

    pub async fn request_json(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> JsonResponse {
        let request = self.client.request(method, self.api_url(path));
        let request = match body {
            Some(body) => request.json(&body),
            None => request,
        };
        let response = request.send().await.expect("send the API request");
        let status = response.status();
        let text = response.text().await.expect("read the API response");
        let body = if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or(Value::String(text))
        };
        JsonResponse { status, body }
    }

    pub async fn get(&self, path: &str) -> JsonResponse {
        self.request_json(Method::GET, path, None).await
    }

    pub async fn post(&self, path: &str, body: Value) -> JsonResponse {
        self.request_json(Method::POST, path, Some(body)).await
    }

    pub async fn put(&self, path: &str, body: Value) -> JsonResponse {
        self.request_json(Method::PUT, path, Some(body)).await
    }

    pub async fn patch(&self, path: &str, body: Value) -> JsonResponse {
        self.request_json(Method::PATCH, path, Some(body)).await
    }

    pub async fn delete(&self, path: &str) -> JsonResponse {
        self.request_json(Method::DELETE, path, None).await
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> JsonResponse {
        self.post(
            "/auth/session",
            json!({ "email": email, "password": password }),
        )
        .await
    }

    pub async fn configure_ai_provider(&self, provider: &str) -> JsonResponse {
        self.put(
            "/ai_configuration",
            json!({ "provider": provider, "api_key": AI_PROVIDER_API_KEY }),
        )
        .await
    }

    pub async fn configure_live_ai_provider(&self, provider: &str) {
        let configured = self.configure_ai_provider(provider).await;
        assert_eq!(
            configured.status,
            StatusCode::OK,
            "AI configuration failed: {}",
            configured.body
        );
        assert_eq!(
            configured.body["status"], "live",
            "the {provider} provider is not live: {}",
            configured.body
        );
    }
}

enum SingleSignOn {
    Disabled,
    MockProviders,
}

enum SelfServeTenants {
    FlagUnset,
    Enabled,
    EnabledWithLimit { limit: u64 },
}

#[derive(Clone, Default)]
pub struct DeploymentFlags {
    pub is_password_sign_in_off: bool,
    pub is_password_administrator_seeding_off: bool,
    pub is_default_tenant_off: bool,
    pub is_self_serve_tenant_creation_on: bool,
    pub platform_administrator_emails: Option<String>,
    pub disabled_connection_types: Option<String>,
    pub is_internal_address_connections_off: bool,
    pub is_background_suggested_question_refresh_off: bool,
    pub is_rate_limiting_on: bool,
    pub is_proxy_client_address_trusted: bool,
    pub rate_limits: Option<RateLimitSettings>,
    pub tenant_member_limit: Option<u64>,
}

impl DeploymentFlags {
    pub fn passwordless(platform_administrator_emails: &str) -> Self {
        Self {
            is_password_sign_in_off: true,
            is_password_administrator_seeding_off: true,
            is_default_tenant_off: true,
            is_self_serve_tenant_creation_on: true,
            platform_administrator_emails: Some(platform_administrator_emails.to_string()),
            ..Self::default()
        }
    }
}

enum EmailDelivery<'a> {
    Disabled,
    MailServer,
    RefusingMailServer,
    UnreachableMailServer,
    SilentMailServer {
        silent_mail_server: &'a SilentMailServer,
    },
}

pub struct RedeployedApp {
    pub admin: UserSession,
    server_task: JoinHandle<()>,
}

impl Drop for RedeployedApp {
    fn drop(&mut self) {
        self.server_task.abort();
    }
}

pub struct TestApp {
    pub admin: UserSession,
    pub fake_llm: FakeLlmServer,
    pub app_state: AppState,
    pub tenant_id: Uuid,
    server_task: JoinHandle<()>,
    sqlite_demo_databases: Mutex<Vec<SqliteDemoDatabase>>,
    postgres_sample_databases: Mutex<Vec<PostgresDatabase>>,
    _metadata_database: MetadataDatabase,
}

impl Deref for TestApp {
    type Target = UserSession;

    fn deref(&self) -> &UserSession {
        &self.admin
    }
}

impl TestApp {
    pub async fn spawn() -> Self {
        Self::spawn_with(SingleSignOn::Disabled, EmailDelivery::Disabled).await
    }

    pub async fn spawn_with_single_sign_on() -> Self {
        Self::spawn_with(SingleSignOn::MockProviders, EmailDelivery::Disabled).await
    }

    pub async fn spawn_with_email() -> Self {
        Self::spawn_with(SingleSignOn::Disabled, EmailDelivery::MailServer).await
    }

    pub async fn spawn_with_email_and_single_sign_on() -> Self {
        Self::spawn_with(SingleSignOn::MockProviders, EmailDelivery::MailServer).await
    }

    pub async fn spawn_with_refusing_mail_server() -> Self {
        Self::spawn_with(SingleSignOn::Disabled, EmailDelivery::RefusingMailServer).await
    }

    pub async fn spawn_with_unreachable_mail_server() -> Self {
        Self::spawn_with(SingleSignOn::Disabled, EmailDelivery::UnreachableMailServer).await
    }

    pub async fn spawn_with_silent_mail_server(silent_mail_server: &SilentMailServer) -> Self {
        Self::spawn_with(
            SingleSignOn::Disabled,
            EmailDelivery::SilentMailServer { silent_mail_server },
        )
        .await
    }

    pub async fn spawn_with_self_serve_tenants() -> Self {
        Self::spawn_with_settings(
            SingleSignOn::Disabled,
            EmailDelivery::Disabled,
            SelfServeTenants::Enabled,
        )
        .await
    }

    pub async fn spawn_with_self_serve_tenant_limit(limit: u64) -> Self {
        Self::spawn_with_settings(
            SingleSignOn::Disabled,
            EmailDelivery::Disabled,
            SelfServeTenants::EnabledWithLimit { limit },
        )
        .await
    }

    pub async fn spawn_with_deployment_flags(deployment_flags: DeploymentFlags) -> Self {
        Self::spawn_with_every_setting(
            SingleSignOn::MockProviders,
            EmailDelivery::MailServer,
            SelfServeTenants::FlagUnset,
            deployment_flags,
        )
        .await
    }

    async fn spawn_with(single_sign_on: SingleSignOn, email_delivery: EmailDelivery<'_>) -> Self {
        Self::spawn_with_settings(single_sign_on, email_delivery, SelfServeTenants::FlagUnset).await
    }

    async fn spawn_with_settings(
        single_sign_on: SingleSignOn,
        email_delivery: EmailDelivery<'_>,
        self_serve_tenants: SelfServeTenants,
    ) -> Self {
        Self::spawn_with_every_setting(
            single_sign_on,
            email_delivery,
            self_serve_tenants,
            DeploymentFlags::default(),
        )
        .await
    }

    async fn spawn_with_every_setting(
        single_sign_on: SingleSignOn,
        email_delivery: EmailDelivery<'_>,
        self_serve_tenants: SelfServeTenants,
        deployment_flags: DeploymentFlags,
    ) -> Self {
        let metadata_database = MetadataDatabase::create().await;
        let fake_llm = FakeLlmServer::start(&manifest_path(FAKE_LLM_FIXTURES_RELATIVE_PATH))
            .await
            .expect("start the fake LLM server");
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind the application listener");
        let base_url = format!(
            "http://{}",
            listener.local_addr().expect("read the application address")
        );
        let oidc_providers = match single_sign_on {
            SingleSignOn::Disabled => Vec::new(),
            SingleSignOn::MockProviders => mock_identity_providers(&base_url).await,
        };
        let email_configuration = match email_delivery {
            EmailDelivery::Disabled => None,
            EmailDelivery::MailServer => {
                Some(shared_mail_server().await.email_configuration(&base_url))
            }
            EmailDelivery::RefusingMailServer => Some(
                shared_refusing_mail_server()
                    .await
                    .email_configuration(&base_url),
            ),
            EmailDelivery::UnreachableMailServer => {
                Some(unreachable_email_configuration(&base_url))
            }
            EmailDelivery::SilentMailServer { silent_mail_server } => {
                Some(silent_mail_server.email_configuration(&base_url))
            }
        };
        let app_state = AppState::from_configurations(
            test_configurations(
                &metadata_database,
                &fake_llm,
                oidc_providers,
                email_configuration,
                self_serve_tenants,
                &deployment_flags,
            )
            .expect("build the test configuration"),
        )
        .await
        .expect("connect to the metadata database");
        sopho::initialize(&app_state)
            .await
            .expect("initialize the application");
        let default_tenant_id = default_tenant_id(&app_state).await;
        let tenant_id = default_tenant_id.unwrap_or(NO_DEFAULT_TENANT_ID);
        let router = sopho::build_app(app_state.clone());
        let server_task = tokio::spawn(async move {
            axum::serve(listener, sopho::into_service_with_client_address(router))
                .await
                .expect("serve the application");
        });
        Self {
            admin: UserSession::new(&base_url, default_tenant_id),
            fake_llm,
            app_state,
            tenant_id,
            server_task,
            sqlite_demo_databases: Mutex::new(Vec::new()),
            postgres_sample_databases: Mutex::new(Vec::new()),
            _metadata_database: metadata_database,
        }
    }

    pub async fn deliver_due_emails(&self) {
        sopho::deliver_due_emails(&self.app_state)
            .await
            .expect("deliver the queued emails");
    }

    pub async fn restart_initialization(&self) {
        sopho::initialize(&self.app_state)
            .await
            .expect("re-run application initialization");
    }

    pub async fn initialize_again_with_deployment_flags(
        &self,
        deployment_flags: DeploymentFlags,
    ) -> anyhow::Result<()> {
        let configurations = test_configurations(
            &self._metadata_database,
            &self.fake_llm,
            mock_identity_providers(self.admin.base_url()).await,
            None,
            SelfServeTenants::FlagUnset,
            &deployment_flags,
        )?;
        let app_state = AppState::from_configurations(configurations).await?;
        sopho::initialize(&app_state).await
    }

    pub async fn redeploy_with_deployment_flags(
        &self,
        deployment_flags: DeploymentFlags,
    ) -> RedeployedApp {
        let configurations = test_configurations(
            &self._metadata_database,
            &self.fake_llm,
            Vec::new(),
            None,
            SelfServeTenants::FlagUnset,
            &deployment_flags,
        )
        .expect("build the redeployed configuration");
        let app_state = AppState::from_configurations(configurations)
            .await
            .expect("connect the redeployed application to the metadata database");
        sopho::initialize(&app_state)
            .await
            .expect("initialize the redeployed application");
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind the redeployed application listener");
        let base_url = format!(
            "http://{}",
            listener
                .local_addr()
                .expect("read the redeployed application address")
        );
        let router = sopho::build_app(app_state);
        let server_task = tokio::spawn(async move {
            axum::serve(listener, sopho::into_service_with_client_address(router))
                .await
                .expect("serve the redeployed application");
        });
        let admin = UserSession::new(&base_url, Some(self.tenant_id));
        let signed_in = admin.sign_in(ADMIN_EMAIL, ADMIN_PASSWORD).await;
        assert_eq!(
            signed_in.status,
            StatusCode::OK,
            "admin login on the redeployed application failed: {}",
            signed_in.body
        );
        RedeployedApp { admin, server_task }
    }

    pub fn session(&self, tenant_id: Option<Uuid>) -> UserSession {
        UserSession::new(self.admin.base_url(), tenant_id)
    }

    pub async fn login_as_admin(&self) {
        let response = self.admin.sign_in(ADMIN_EMAIL, ADMIN_PASSWORD).await;
        assert_eq!(
            response.status,
            StatusCode::OK,
            "admin login failed: {}",
            response.body
        );
    }

    pub async fn create_sqlite_demo_connection(&mut self) -> Value {
        self.create_sqlite_demo_connection_as(&self.admin).await
    }

    pub async fn create_sqlite_demo_connection_as(&self, session: &UserSession) -> Value {
        self.create_named_sqlite_demo_connection_as(session, SQLITE_DEMO_CONNECTION_NAME)
            .await
    }

    pub async fn create_named_sqlite_demo_connection_as(
        &self,
        session: &UserSession,
        name: &str,
    ) -> Value {
        let demo_database = copy_sqlite_demo_database();
        let payload = json!({
            "name": name,
            "source_type": "SQLITE",
            "database": demo_database.path().display().to_string(),
            "port": null,
        });
        self.sqlite_demo_databases
            .lock()
            .expect("lock the SQLite demo databases")
            .push(demo_database);
        create_connection(session, payload).await
    }

    pub async fn create_postgres_sample_connection(&mut self) -> Value {
        let sample_database = create_postgres_sample_database().await;
        let server = shared_postgres_server().await;
        let payload = json!({
            "name": POSTGRES_SAMPLE_CONNECTION_NAME,
            "source_type": "POSTGRESQL",
            "host": server.host,
            "port": server.port,
            "username": server.username,
            "password": server.password,
            "database": sample_database.name,
            "schema": SAMPLE_DATABASE_SCHEMA,
        });
        self.postgres_sample_databases
            .lock()
            .expect("lock the Postgres sample databases")
            .push(sample_database);
        create_connection(&self.admin, payload).await
    }
}

async fn create_connection(session: &UserSession, payload: Value) -> Value {
    let response = session.post("/connection", payload).await;
    assert_eq!(
        response.status,
        StatusCode::CREATED,
        "connection creation failed: {}",
        response.body
    );
    response.body
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.server_task.abort();
        if std::thread::panicking() {
            return;
        }
        let unmatched_requests = self.fake_llm.unmatched_requests();
        assert!(
            unmatched_requests.is_empty(),
            "the fake LLM could not answer {} request(s): {:#?}",
            unmatched_requests.len(),
            unmatched_requests
                .iter()
                .map(|request| request.body.chars().take(600).collect::<String>())
                .collect::<Vec<_>>()
        );
    }
}

async fn default_tenant_id(app_state: &AppState) -> Option<Uuid> {
    let query = Query::select()
        .column(Alias::new("id"))
        .from(Alias::new("tenant"))
        .to_owned();
    let backend = app_state.database_connection.get_database_backend();
    let tenant_rows = app_state
        .database_connection
        .query_all(backend.build(&query))
        .await
        .expect("read the tenants");
    assert!(
        tenant_rows.len() <= 1,
        "initialization creates at most the default tenant"
    );
    tenant_rows.first().map(|tenant_row| {
        tenant_row
            .try_get::<Uuid>("", "id")
            .expect("read the default tenant id")
    })
}

fn test_configurations(
    metadata_database: &MetadataDatabase,
    fake_llm: &FakeLlmServer,
    oidc_providers: Vec<OidcProviderConfiguration>,
    email_configuration: Option<EmailConfiguration>,
    self_serve_tenants: SelfServeTenants,
    deployment_flags: &DeploymentFlags,
) -> anyhow::Result<Configurations> {
    let configurations_builder = Configurations::builder()
        .database_url(metadata_database.url())
        .port(0)
        .frontend_dir(FRONTEND_DIRECTORY_FOR_API_TESTS)
        .environment(ENVIRONMENT)
        .cookie_secure(false)
        .encryption_key(ENCRYPTION_KEY)
        .admin_username(ADMIN_USERNAME)
        .admin_password(ADMIN_PASSWORD)
        .admin_email(ADMIN_EMAIL)
        .admin_full_name(ADMIN_FULL_NAME)
        .anthropic_base_url(fake_llm.anthropic_base_url())
        .openai_base_url(fake_llm.openai_base_url())
        .oidc_providers(oidc_providers);
    let configurations_builder = match self_serve_tenants {
        SelfServeTenants::FlagUnset => configurations_builder,
        SelfServeTenants::Enabled => {
            configurations_builder.feature_flag(FeatureFlag::SelfServeTenantCreation, true)
        }
        SelfServeTenants::EnabledWithLimit { limit } => configurations_builder
            .feature_flag(FeatureFlag::SelfServeTenantCreation, true)
            .self_serve_tenant_limit(limit),
    };
    let configurations_builder = configurations_builder
        .feature_flag(
            FeatureFlag::PasswordSignIn,
            !deployment_flags.is_password_sign_in_off,
        )
        .feature_flag(
            FeatureFlag::SeedPasswordAdministrator,
            !deployment_flags.is_password_administrator_seeding_off,
        )
        .feature_flag(
            FeatureFlag::DefaultTenant,
            !deployment_flags.is_default_tenant_off,
        );
    let configurations_builder = match deployment_flags.is_self_serve_tenant_creation_on {
        true => configurations_builder.feature_flag(FeatureFlag::SelfServeTenantCreation, true),
        false => configurations_builder,
    };
    let configurations_builder = configurations_builder
        .feature_flag(
            FeatureFlag::InternalAddressConnections,
            !deployment_flags.is_internal_address_connections_off,
        )
        .feature_flag(
            FeatureFlag::BackgroundSuggestedQuestionRefresh,
            !deployment_flags.is_background_suggested_question_refresh_off,
        );
    let configurations_builder = configurations_builder
        .feature_flag(
            FeatureFlag::RateLimiting,
            deployment_flags.is_rate_limiting_on,
        )
        .feature_flag(
            FeatureFlag::TrustProxyClientAddress,
            deployment_flags.is_proxy_client_address_trusted,
        );
    let configurations_builder = match deployment_flags.rate_limits {
        Some(rate_limits) => configurations_builder.rate_limits(rate_limits),
        None => configurations_builder,
    };
    let configurations_builder = match deployment_flags.tenant_member_limit {
        Some(tenant_member_limit) => {
            configurations_builder.tenant_member_limit(tenant_member_limit)
        }
        None => configurations_builder,
    };
    let configurations_builder = match &deployment_flags.disabled_connection_types {
        Some(disabled_connection_types) => {
            configurations_builder.disabled_connection_types(disabled_connection_types)
        }
        None => configurations_builder,
    };
    let configurations_builder = match &deployment_flags.platform_administrator_emails {
        Some(platform_administrator_emails) => {
            configurations_builder.platform_administrator_emails(platform_administrator_emails)
        }
        None => configurations_builder,
    };
    Ok(match email_configuration {
        Some(email_configuration) => configurations_builder.email(email_configuration),
        None => configurations_builder,
    }
    .build()?)
}
