use crate::support::constants::{
    ADMIN_EMAIL, ADMIN_FULL_NAME, ADMIN_PASSWORD, ADMIN_USERNAME, AI_PROVIDER_API_KEY, API_PREFIX,
    ENCRYPTION_KEY, ENVIRONMENT, FAKE_LLM_FIXTURES_RELATIVE_PATH, FRONTEND_DIRECTORY_FOR_API_TESTS,
    POSTGRES_SAMPLE_CONNECTION_NAME, SAMPLE_DATABASE_SCHEMA, SQLITE_DEMO_CONNECTION_NAME,
    TENANT_PATH_SEGMENT, TENANT_SCOPED_ROUTE_GROUPS,
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
use sopho::{AppState, Configurations, EmailConfiguration, OidcProviderConfiguration};
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

enum EmailDelivery<'a> {
    Disabled,
    MailServer,
    RefusingMailServer,
    UnreachableMailServer,
    SilentMailServer {
        silent_mail_server: &'a SilentMailServer,
    },
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

    async fn spawn_with(single_sign_on: SingleSignOn, email_delivery: EmailDelivery<'_>) -> Self {
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
        let app_state = AppState::from_configurations(test_configurations(
            &metadata_database,
            &fake_llm,
            oidc_providers,
            email_configuration,
        ))
        .await
        .expect("connect to the metadata database");
        sopho::initialize(&app_state)
            .await
            .expect("initialize the application");
        let tenant_id = default_tenant_id(&app_state).await;
        let router = sopho::build_app(app_state.clone());
        let server_task = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("serve the application");
        });
        Self {
            admin: UserSession::new(&base_url, Some(tenant_id)),
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

async fn default_tenant_id(app_state: &AppState) -> Uuid {
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
    assert_eq!(
        tenant_rows.len(),
        1,
        "initialization creates exactly the default tenant"
    );
    tenant_rows[0]
        .try_get::<Uuid>("", "id")
        .expect("read the default tenant id")
}

fn test_configurations(
    metadata_database: &MetadataDatabase,
    fake_llm: &FakeLlmServer,
    oidc_providers: Vec<OidcProviderConfiguration>,
    email_configuration: Option<EmailConfiguration>,
) -> Configurations {
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
    match email_configuration {
        Some(email_configuration) => configurations_builder.email(email_configuration),
        None => configurations_builder,
    }
    .build()
    .expect("build the test configuration")
}
