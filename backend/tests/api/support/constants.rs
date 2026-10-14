pub const API_PREFIX: &str = "/api/v1";

pub const ADMIN_USERNAME: &str = "admin";
pub const ADMIN_PASSWORD: &str = "test-admin-password";
pub const ADMIN_EMAIL: &str = "admin@sopho.test";
pub const ADMIN_FULL_NAME: &str = "Test Admin";
pub const ENCRYPTION_KEY: &str = "integration-test-encryption-key";
pub const ENVIRONMENT: &str = "production";
pub const AI_PROVIDER_API_KEY: &str = "fake-llm-api-key";

pub const ACCESS_TOKEN_COOKIE: &str = "ACCESS_TOKEN";
pub const REFRESH_TOKEN_COOKIE: &str = "REFRESH_TOKEN";

pub const TEST_DATABASE_VARIABLE: &str = "TEST_DB";
pub const TEST_POSTGRES_URL_VARIABLE: &str = "TEST_POSTGRES_URL";

pub const POSTGRES_IMAGE_NAME: &str = "postgres";
pub const POSTGRES_IMAGE_TAG: &str = "17";
pub const POSTGRES_CONTAINER_NAME: &str = "sopho-test-postgres";
pub const POSTGRES_PORT: u16 = 5432;
pub const POSTGRES_USER: &str = "postgres";
pub const POSTGRES_PASSWORD: &str = "postgres";
pub const POSTGRES_ADMIN_DATABASE: &str = "postgres";
pub const POSTGRES_READY_MESSAGE: &str = "database system is ready to accept connections";
pub const POSTGRES_STARTUP_LOCK_FILE_NAME: &str = "sopho-test-postgres.lock";
pub const POSTGRES_READY_TIMEOUT_SECONDS: u64 = 60;
pub const POSTGRES_READY_POLL_MILLISECONDS: u64 = 250;

pub const METADATA_DATABASE_PREFIX: &str = "test";
pub const SAMPLE_DATABASE_PREFIX: &str = "sample";
pub const SAMPLE_DATABASE_SCHEMA: &str = "public";

pub const DEMO_DATABASE_RELATIVE_PATH: &str = "demo/ecommerce.db";
pub const DEMO_DATABASE_FILE_NAME: &str = "ecommerce.db";
pub const SAMPLE_DATA_RELATIVE_PATH: &str = "tests/api/fixtures/sample_data.sql";
pub const FAKE_LLM_FIXTURES_RELATIVE_PATH: &str = "fake-llm/fixtures";
pub const FRONTEND_DIRECTORY_FOR_API_TESTS: &str = "frontend-is-not-served-in-api-tests";

pub const TOP_CATEGORIES_QUERY: &str = "SELECT product_category_name AS category, COUNT(*) AS product_count FROM products GROUP BY product_category_name ORDER BY product_count DESC LIMIT 5";
pub const ORDER_COUNT_QUERY: &str = "SELECT COUNT(*) AS order_count FROM orders";
pub const DEMO_ORDER_COUNT: i64 = 10000;

pub const SQLITE_DEMO_CONNECTION_NAME: &str = "Ecommerce demo copy";
pub const POSTGRES_SAMPLE_CONNECTION_NAME: &str = "Postgres sample";

pub const ORDER_STATUS_QUESTION: &str = "How many orders are in each status?";
pub const ORDER_STATUS_QUERY: &str = "SELECT order_status, COUNT(*) AS order_count FROM orders GROUP BY order_status ORDER BY order_count DESC, order_status";
pub const PERSISTED_EVENT_TIMEOUT_SECONDS: u64 = 30;
pub const PERSISTED_EVENT_POLL_MILLISECONDS: u64 = 100;
