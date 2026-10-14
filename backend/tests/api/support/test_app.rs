use crate::support::constants::{
    ADMIN_EMAIL, ADMIN_FULL_NAME, ADMIN_PASSWORD, ADMIN_USERNAME, AI_PROVIDER_API_KEY, API_PREFIX,
    ENCRYPTION_KEY, ENVIRONMENT, FAKE_LLM_FIXTURES_RELATIVE_PATH, FRONTEND_DIRECTORY_FOR_API_TESTS,
    POSTGRES_SAMPLE_CONNECTION_NAME, SAMPLE_DATABASE_SCHEMA, SQLITE_DEMO_CONNECTION_NAME,
};
use crate::support::data_sources::{
    SqliteDemoDatabase, copy_sqlite_demo_database, create_postgres_sample_database, manifest_path,
};
use crate::support::metadata_database::MetadataDatabase;
use crate::support::postgres_server::{PostgresDatabase, shared_postgres_server};
use fake_llm::FakeLlmServer;
use reqwest::{Client, Method, StatusCode};
use serde_json::{Value, json};
use sopho::{AppState, Configurations};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct JsonResponse {
    pub status: StatusCode,
    pub body: Value,
}

pub struct TestApp {
    pub client: Client,
    pub fake_llm: FakeLlmServer,
    pub app_state: AppState,
    base_url: String,
    server_task: JoinHandle<()>,
    _sqlite_demo_databases: Vec<SqliteDemoDatabase>,
    _postgres_sample_databases: Vec<PostgresDatabase>,
    _metadata_database: MetadataDatabase,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let metadata_database = MetadataDatabase::create().await;
        let fake_llm = FakeLlmServer::start(&manifest_path(FAKE_LLM_FIXTURES_RELATIVE_PATH))
            .await
            .expect("start the fake LLM server");
        let app_state =
            AppState::from_configurations(test_configurations(&metadata_database, &fake_llm))
                .await
                .expect("connect to the metadata database");
        sopho::initialize(&app_state)
            .await
            .expect("initialize the application");
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind the application listener");
        let base_url = format!(
            "http://{}",
            listener.local_addr().expect("read the application address")
        );
        let router = sopho::build_app(app_state.clone());
        let server_task = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("serve the application");
        });
        Self {
            client: Client::builder()
                .cookie_store(true)
                .build()
                .expect("build the HTTP client"),
            fake_llm,
            app_state,
            base_url,
            server_task,
            _sqlite_demo_databases: Vec::new(),
            _postgres_sample_databases: Vec::new(),
            _metadata_database: metadata_database,
        }
    }

    pub async fn restart_initialization(&self) {
        sopho::initialize(&self.app_state)
            .await
            .expect("re-run application initialization");
    }

    pub fn api_url(&self, path: &str) -> String {
        format!("{}{API_PREFIX}{path}", self.base_url)
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

    pub async fn login_as_admin(&self) {
        let response = self
            .post(
                "/auth/session",
                json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }),
            )
            .await;
        assert_eq!(
            response.status,
            StatusCode::OK,
            "admin login failed: {}",
            response.body
        );
    }

    pub async fn create_sqlite_demo_connection(&mut self) -> Value {
        let demo_database = copy_sqlite_demo_database();
        let payload = json!({
            "name": SQLITE_DEMO_CONNECTION_NAME,
            "source_type": "SQLITE",
            "database": demo_database.path().display().to_string(),
            "port": null,
        });
        self._sqlite_demo_databases.push(demo_database);
        self.create_connection(payload).await
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
        self._postgres_sample_databases.push(sample_database);
        self.create_connection(payload).await
    }

    async fn create_connection(&self, payload: Value) -> Value {
        let response = self.post("/connection", payload).await;
        assert_eq!(
            response.status,
            StatusCode::CREATED,
            "connection creation failed: {}",
            response.body
        );
        response.body
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

fn test_configurations(
    metadata_database: &MetadataDatabase,
    fake_llm: &FakeLlmServer,
) -> Configurations {
    Configurations::builder()
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
        .build()
        .expect("build the test configuration")
}
