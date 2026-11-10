use strum::{AsRefStr, Display};

pub const API_PREFIX: &str = "/api/v1";
pub const TENANT_PATH_SEGMENT: &str = "tenant";
pub const TENANT_SCOPED_ROUTE_GROUPS: [&str; 13] = [
    "dashboard",
    "connection",
    "notebook",
    "cell",
    "grant",
    "canvas",
    "search",
    "conversational_analytics",
    "ai_configuration",
    "suggested_question",
    "ai_summary",
    "member",
    "role",
];

pub const ADMIN_USERNAME: &str = "admin";
pub const ADMIN_PASSWORD: &str = "test-admin-password";
pub const ADMIN_EMAIL: &str = "admin@sopho.test";
pub const ADMIN_FULL_NAME: &str = "Test Admin";
pub const ENCRYPTION_KEY: &str = "integration-test-encryption-key";
pub const ENVIRONMENT: &str = "production";
pub const AI_PROVIDER_API_KEY: &str = "fake-llm-api-key";

pub const ACCESS_TOKEN_COOKIE: &str = "ACCESS_TOKEN";
pub const REFRESH_TOKEN_COOKIE: &str = "REFRESH_TOKEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TestEnvironmentVariable {
    TestDb,
    TestPostgresUrl,
}

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

pub const TEST_USER_PASSWORD: &str = "test-user-password";
pub const TEST_USER_EMAIL_DOMAIN: &str = "example.test";

pub const PLATFORM_MEMBER: &str = "MEMBER";
pub const USER_STATUS_ACTIVE: &str = "ACTIVE";
pub const OWNER_ROLE: &str = "OWNER";
pub const ADMIN_ROLE: &str = "ADMIN";
pub const MEMBER_ROLE: &str = "MEMBER";
pub const VIEWER_ROLE: &str = "VIEWER";
pub const AUDITOR_ROLE: &str = "AUDITOR";
pub const ACTIVE_STATUS: &str = "ACTIVE";
pub const SUSPENDED_STATUS: &str = "SUSPENDED";
pub const PRIVATE_VISIBILITY: &str = "PRIVATE";
pub const TENANT_VISIBILITY: &str = "TENANT";
pub const CANVAS_RESOURCE: &str = "CANVAS";
pub const CONVERSATION_RESOURCE: &str = "CONVERSATION";
pub const CONNECTION_RESOURCE: &str = "CONNECTION";
pub const USER_PRINCIPAL: &str = "USER";
pub const VIEWER_ACCESS: &str = "VIEWER";
pub const EDITOR_ACCESS: &str = "EDITOR";
pub const SYSTEM_ACTOR: &str = "SYSTEM";
pub const HUMAN_ACTOR: &str = "HUMAN";
pub const AI_ACTOR: &str = "AI";

pub const MOCK_IDENTITY_IMAGE_NAME: &str = "ghcr.io/navikt/mock-oauth2-server";
pub const MOCK_IDENTITY_IMAGE_TAG: &str = "2.1.10";
pub const MOCK_IDENTITY_CONTAINER_NAME: &str = "sopho-test-oidc";
pub const MOCK_IDENTITY_PORT: u16 = 8080;
pub const MOCK_IDENTITY_CONFIGURATION: &str = r#"{"interactiveLogin":true}"#;
pub const MOCK_IDENTITY_STARTUP_LOCK_FILE_NAME: &str = "sopho-test-oidc.lock";
pub const MOCK_IDENTITY_READY_TIMEOUT_SECONDS: u64 = 60;
pub const MOCK_IDENTITY_READY_POLL_MILLISECONDS: u64 = 250;
pub const MOCK_IDENTITY_DISCOVERY_PATH: &str = ".well-known/openid-configuration";
pub const MOCK_IDENTITY_CLIENT_ID: &str = "sopho-api-tests";
pub const MOCK_IDENTITY_CLIENT_SECRET: &str = "sopho-api-tests-secret";
pub const MOCK_IDENTITY_SUBJECT_FIELD: &str = "username";
pub const MOCK_IDENTITY_CLAIMS_FIELD: &str = "claims";
pub const MAIL_SERVER_IMAGE_NAME: &str = "axllent/mailpit";
pub const MAIL_SERVER_IMAGE_TAG: &str = "v1.31.4";
pub const MAIL_SERVER_CONTAINER_NAME: &str = "sopho-test-mail-accepting";
pub const REFUSING_MAIL_SERVER_CONTAINER_NAME: &str = "sopho-test-mail-refusing";
pub const MAIL_SERVER_CONTAINER_LABEL: &str = "sopho.test-mail-server";
pub const MAIL_SERVER_SMTP_PORT: u16 = 1025;
pub const MAIL_SERVER_API_PORT: u16 = 8025;
pub const MAIL_SERVER_MESSAGE_LIMIT: &str = "5000";
pub const MAIL_SERVER_REFUSE_EVERY_RECIPIENT: &str = "^refuse-everyone$";
pub const MAIL_SERVER_STARTUP_LOCK_FILE_NAME: &str = "sopho-test-mailpit.lock";
pub const MAIL_SERVER_READY_TIMEOUT_SECONDS: u64 = 60;
pub const MAIL_SERVER_READY_POLL_MILLISECONDS: u64 = 250;
pub const MAIL_SERVER_TIMEOUT_SECONDS: u64 = 30;
pub const SILENT_MAIL_SERVER_TIMEOUT_SECONDS: u64 = 1;
pub const SILENT_MAIL_SERVER_WAIT_SECONDS: u64 = 15;
pub const SILENT_MAIL_SERVER_DELIVERY_DEADLINE_SECONDS: u64 = 15;
pub const SMTP_TIMED_OUT_MESSAGE: &str = "The mail server did not answer in time";
pub const EMAIL_SENDER: &str = "Sopho <no-reply@sopho.test>";
pub const INVITATION_LINK_PATH: &str = "/invitation#token=";
pub const EMAIL_OUTBOX_TABLE: &str = "email_outbox";
pub const ACCEPTED_INVITATION_STATUS: &str = "ACCEPTED";
pub const INVITATION_LINK_INVALID_CODE: &str = "INVITATION_LINK_INVALID";
pub const INVITATION_FOR_ANOTHER_ACCOUNT_CODE: &str = "INVITATION_FOR_ANOTHER_ACCOUNT";
pub const ACCOUNT_EXISTS_CODE: &str = "ACCOUNT_EXISTS";
pub const FULL_NAME_REQUIRED_CODE: &str = "FULL_NAME_REQUIRED";
pub const PASSWORD_TOO_SHORT_CODE: &str = "PASSWORD_TOO_SHORT";
pub const PASSWORD_CONTAINS_SERVICE_NAME_CODE: &str = "PASSWORD_CONTAINS_SERVICE_NAME";
pub const NEW_ACCOUNT_PASSWORD: &str = "correct horse battery staple";
pub const RESET_LINK_PATH: &str = "/reset-password#token=";
pub const RESET_REQUESTED_MESSAGE: &str = "If an account uses this email, we've sent a link";
pub const PASSWORD_RESET_EMAIL_KIND: &str = "PASSWORD_RESET";
pub const RESET_EMAILS_PER_HOUR: usize = 3;
pub const SIMULTANEOUS_RESET_REQUESTS: usize = 10;
pub const NO_PASSWORD_EMAIL_KIND: &str = "NO_PASSWORD";
pub const PASSWORD_CHANGED_EMAIL_KIND: &str = "PASSWORD_CHANGED";
pub const INVALID_RESET_LINK_CODE: &str = "INVALID_RESET_LINK";
pub const WRONG_CURRENT_PASSWORD_CODE: &str = "WRONG_CURRENT_PASSWORD";
pub const TOO_MANY_SIGN_IN_ATTEMPTS_CODE: &str = "TOO_MANY_SIGN_IN_ATTEMPTS";
pub const ACCOUNT_DEACTIVATED_CODE: &str = "ACCOUNT_DEACTIVATED";
pub const USER_STATUS_DEACTIVATED: &str = "DEACTIVATED";
pub const QUEUED_EMAIL_WAIT_SECONDS: u64 = 10;
pub const SIGN_IN_METHOD_ADDED_EMAIL_KIND: &str = "SIGN_IN_METHOD_ADDED";
pub const SIGN_IN_METHOD_REMOVED_EMAIL_KIND: &str = "SIGN_IN_METHOD_REMOVED";
pub const LAST_SIGN_IN_METHOD_CODE: &str = "LAST_SIGN_IN_METHOD";
pub const CONNECT_RESULT_LOCATION_PREFIX: &str = "/profile?connect_result=";
pub const REVOKED_INVITATION_STATUS: &str = "REVOKED";
pub const CANNOT_DEACTIVATE_YOURSELF_CODE: &str = "CANNOT_DEACTIVATE_YOURSELF";
pub const APPOINTEE_DEACTIVATED_CODE: &str = "APPOINTEE_DEACTIVATED";
pub const EMAIL_NOT_RETRYABLE_CODE: &str = "EMAIL_NOT_RETRYABLE";
pub const NO_PASSWORD_CODE: &str = "NO_PASSWORD";
pub const QUEUED_EMAIL_POLL_MILLISECONDS: u64 = 50;
pub const EMAIL_PENDING_STATUS: &str = "PENDING";
pub const EMAIL_SENDING_STATUS: &str = "SENDING";
pub const EMAIL_SENT_STATUS: &str = "SENT";
pub const EMAIL_FAILED_STATUS: &str = "FAILED";
pub const EMAIL_CANCELLED_STATUS: &str = "CANCELLED";
pub const INVITATION_EMAIL_KIND: &str = "INVITATION";
pub const EXPIRED_EMAIL_MESSAGE: &str = "Its links expired before it could be sent";
pub const UNDECRYPTABLE_EMAIL_MESSAGE: &str = "The email could not be decrypted";
pub const RESEND_NEEDS_EMAIL_MESSAGE: &str =
    "Resending an invitation needs email. Configure SOPHO_SMTP_HOST first";
pub const INVITATIONS_UNAVAILABLE_MESSAGE: &str = "Inviting people needs email or single sign-on. Configure SOPHO_SMTP_HOST or SOPHO_OIDC_PROVIDERS before inviting people";
pub const SINGLE_SIGN_ON_PROVIDER: &str = "test";
pub const SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME: &str = "Test SSO";
pub const SINGLE_SIGN_ON_PROVIDER_ISSUER: &str = "default";
pub const OTHER_SINGLE_SIGN_ON_PROVIDER: &str = "other";
pub const OTHER_SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME: &str = "Other SSO";
pub const OTHER_SINGLE_SIGN_ON_PROVIDER_ISSUER: &str = "other";
pub const OIDC_STATE_COOKIE: &str = "OIDC_STATE";
pub const AFTER_SIGN_IN_LOCATION: &str = "/";
pub const SIGN_IN_ERROR_LOCATION_PREFIX: &str = "/signin?error=";
pub const SSO_FAILED: &str = "sso_failed";
pub const EMAIL_NOT_VERIFIED: &str = "email_not_verified";
pub const SSO_EXPIRED: &str = "sso_expired";
pub const PASSWORD_ACCOUNT: &str = "password_account";
pub const ACCOUNT_DEACTIVATED_SIGN_IN_ERROR: &str = "account_deactivated";
pub const LOGIN_ATTEMPT_TABLE: &str = "oidc_login_attempt";
pub const TENANT_CREATE_PERMISSION: &str = "tenant:create";
pub const USER_MANAGE_PERMISSION: &str = "user:manage";
pub const TENANT_APPOINT_OWNER_PERMISSION: &str = "tenant:appoint_owner";
pub const EMAIL_MANAGE_PERMISSION: &str = "email:manage";

pub const DEMO_CANVAS_NAME: &str = "Ecommerce Sales";
pub const DEMO_CONNECTION_NAME: &str = "Ecommerce Sales Demo";
pub const LIVE_AI_PROVIDER: &str = "anthropic";
pub const ASSISTANT_SENDER: &str = "ASSISTANT";
pub const UNKNOWN_CREATED_BY_FILTER_MESSAGE: &str = "created_by only accepts \"me\"";
pub const SYSTEM_ROLES: [&str; 4] = [OWNER_ROLE, ADMIN_ROLE, MEMBER_ROLE, VIEWER_ROLE];
pub const ATTRIBUTED_TABLES: [&str; 14] = [
    "canvas",
    "notebook",
    "cell",
    "dashboard",
    "connection",
    "conversation",
    "conversation_message",
    "conversation_message_content",
    "ai_summary",
    "suggested_question",
    "ai_configuration",
    "tenant",
    "tenant_membership",
    "role",
];
pub const ATTRIBUTION_COLUMNS: [&str; 4] = [
    "created_by_actor",
    "created_by_user_id",
    "modified_by_actor",
    "modified_by_user_id",
];
pub const SQLITE_WRITE_ORDER_TRIGGERS: &str = "
    CREATE TABLE write_log (write_sequence INTEGER PRIMARY KEY AUTOINCREMENT, table_name TEXT NOT NULL);
    CREATE TRIGGER log_canvas_update AFTER UPDATE ON canvas BEGIN INSERT INTO write_log (table_name) VALUES ('canvas'); END;
    CREATE TRIGGER log_canvas_delete AFTER DELETE ON canvas BEGIN INSERT INTO write_log (table_name) VALUES ('canvas'); END;
    CREATE TRIGGER log_cell_insert AFTER INSERT ON cell BEGIN INSERT INTO write_log (table_name) VALUES ('cell'); END;
    CREATE TRIGGER log_cell_update AFTER UPDATE ON cell BEGIN INSERT INTO write_log (table_name) VALUES ('cell'); END;
    CREATE TRIGGER log_cell_delete AFTER DELETE ON cell BEGIN INSERT INTO write_log (table_name) VALUES ('cell'); END;
    CREATE TRIGGER log_notebook_delete AFTER DELETE ON notebook BEGIN INSERT INTO write_log (table_name) VALUES ('notebook'); END;
    CREATE TRIGGER log_dashboard_update AFTER UPDATE ON dashboard BEGIN INSERT INTO write_log (table_name) VALUES ('dashboard'); END;
    CREATE TRIGGER log_dashboard_delete AFTER DELETE ON dashboard BEGIN INSERT INTO write_log (table_name) VALUES ('dashboard'); END;
";
pub const POSTGRES_WRITE_ORDER_TRIGGERS: &str = "
    CREATE TABLE write_log (write_sequence BIGSERIAL PRIMARY KEY, table_name TEXT NOT NULL);
    CREATE FUNCTION log_write() RETURNS trigger LANGUAGE plpgsql AS $$
    BEGIN
        INSERT INTO write_log (table_name) VALUES (TG_TABLE_NAME);
        RETURN NULL;
    END;
    $$;
    CREATE TRIGGER log_canvas_write AFTER UPDATE OR DELETE ON canvas FOR EACH ROW EXECUTE FUNCTION log_write();
    CREATE TRIGGER log_cell_write AFTER INSERT OR UPDATE OR DELETE ON cell FOR EACH ROW EXECUTE FUNCTION log_write();
    CREATE TRIGGER log_notebook_delete AFTER DELETE ON notebook FOR EACH ROW EXECUTE FUNCTION log_write();
    CREATE TRIGGER log_dashboard_write AFTER UPDATE OR DELETE ON dashboard FOR EACH ROW EXECUTE FUNCTION log_write();
";

pub const INVITATION_TABLE: &str = "tenant_invitation";
pub const PENDING_INVITATION_STATUS: &str = "PENDING";
pub const EXPIRED_INVITATION_STATUS: &str = "EXPIRED";
pub const INVITATION_NOT_FOUND_MESSAGE: &str = "Invitation not found";
pub const INVALID_EMAIL_MESSAGE: &str = "Enter a valid email address";
pub const ALREADY_A_MEMBER_MESSAGE: &str = "This person is already a member of the tenant";
pub const ALREADY_INVITED_MESSAGE: &str = "This email already has a pending invitation";
pub const INVITATION_EXPIRED_MESSAGE: &str = "This invitation has expired";
pub const UNKNOWN_ROLE_MESSAGE: &str = "Unknown role";
pub const ROLE_TOO_POWERFUL_MESSAGE: &str = "You can only grant roles whose permissions you hold";

pub const MEMBER_NOT_FOUND_MESSAGE: &str = "Member not found";
pub const MEMBER_TOO_POWERFUL_MESSAGE: &str =
    "You can only manage members whose role permissions you hold";
pub const LAST_OWNER_MESSAGE: &str = "A tenant needs at least one owner";

pub const ANSWER_QUERY: &str = "SELECT 1 AS answer";
pub const NOTEBOOK_NOT_FOUND_MESSAGE: &str = "Notebook not found";
pub const CONNECTION_NOT_FOUND_MESSAGE: &str = "Connection not found";
pub const CELL_NOT_FOUND_MESSAGE: &str = "Cell not found";
pub const CELL_NOTEBOOK_IS_FIXED_MESSAGE: &str = "A cell cannot be moved to another notebook";
