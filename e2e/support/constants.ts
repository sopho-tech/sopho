import path from "node:path";

export const E2E_DIRECTORY = path.resolve(import.meta.dirname, "..");
export const REPOSITORY_DIRECTORY = path.resolve(E2E_DIRECTORY, "..");

export const SOPHO_PORT = 18080;
export const SOPHO_BASE_URL = `http://127.0.0.1:${SOPHO_PORT}`;
export const SOPHO_BINARY_PATH = path.join(
  REPOSITORY_DIRECTORY,
  "backend",
  "target",
  "debug",
  "sopho",
);
export const FRONTEND_DIST_DIRECTORY = path.join(
  REPOSITORY_DIRECTORY,
  "frontend",
  "dist",
);
export const E2E_DATA_DIRECTORY = path.join(E2E_DIRECTORY, ".e2e-data");
export const METADATA_DATABASE_URL = `sqlite://${path.join(E2E_DATA_DIRECTORY, "sopho.db")}?mode=rwc`;

export const FAKE_LLM_PORT = 18081;
export const FAKE_LLM_BASE_URL = `http://127.0.0.1:${FAKE_LLM_PORT}`;
export const FAKE_LLM_BINARY_PATH = path.join(
  REPOSITORY_DIRECTORY,
  "backend",
  "target",
  "debug",
  "fake-llm",
);
export const FAKE_LLM_FIXTURES_DIRECTORY = path.join(
  REPOSITORY_DIRECTORY,
  "backend",
  "fake-llm",
  "fixtures",
);
export const FAKE_LLM_READY_MESSAGE = /fake LLM listening/;
export const FAKE_LLM_API_KEY = "fake-llm-api-key";

export const MAILPIT_IMAGE = "axllent/mailpit:v1.31.4";
export const MAILPIT_CONTAINER_NAME = "sopho-e2e-mailpit";
export const MAILPIT_SMTP_PORT = 18082;
export const MAILPIT_HTTP_PORT = 18083;
export const MAILPIT_BASE_URL = `http://127.0.0.1:${MAILPIT_HTTP_PORT}`;
export const MAILPIT_READY_PATH = "/livez";
export const EMAIL_SENDER = "Sopho <no-reply@sopho.test>";
export const EMAIL_ARRIVAL_TIMEOUT_MILLISECONDS = 30_000;

export const MOCK_IDENTITY_IMAGE = "ghcr.io/navikt/mock-oauth2-server:2.1.10";
export const MOCK_IDENTITY_CONTAINER_NAME = "sopho-e2e-oidc";
export const MOCK_IDENTITY_PORT = 18084;
export const MOCK_IDENTITY_ISSUER_URL = `http://127.0.0.1:${MOCK_IDENTITY_PORT}/default`;
export const MOCK_IDENTITY_DISCOVERY_PATH = "/.well-known/openid-configuration";
export const MOCK_IDENTITY_CONFIGURATION = '{"interactiveLogin":true}';
export const SINGLE_SIGN_ON_PROVIDER = "e2e";
export const SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME = "E2E SSO";
export const SINGLE_SIGN_ON_CLIENT_ID = "sopho-e2e";
export const SINGLE_SIGN_ON_CLIENT_SECRET = "sopho-e2e-secret";
export const SINGLE_SIGN_ON_REDIRECT_URI = `${SOPHO_BASE_URL}/api/v1/auth/oidc/${SINGLE_SIGN_ON_PROVIDER}/callback`;
export const DOCKER_SHUTDOWN_TIMEOUT_MILLISECONDS = 10_000;

export const INVITED_ROLE = "MEMBER";
export const INVITED_ROLE_DISPLAY_NAME = "Member";
export const REDUCED_MOTION_CONTEXT_OPTIONS = {
  reducedMotion: "reduce",
} as const;
export const INVITEE_EMAIL_DOMAIN = "example.test";
export const INVITEE_PASSWORD = "violet harbour lantern seventeen";
export const RESET_PASSWORD = "copper meadow whistle eleven";
export const INVITATION_LINK_PATH = "/invitation#token=";
export const RESET_LINK_PATH = "/reset-password#token=";
export const INVITATION_SUBJECT = "invited you to";
export const PASSWORD_RESET_SUBJECT = "Reset your Sopho password";
export const RESET_REQUESTED_MESSAGE =
  "If an account uses this email, we've sent a link";
export const MOCK_IDENTITY_SUBJECT_FIELD = "username";
export const MOCK_IDENTITY_CLAIMS_FIELD = "claims";
export const MOCK_IDENTITY_SUBMIT_LABEL = "Sign-in";

export const ADMIN_USERNAME = "admin";
export const ADMIN_PASSWORD = "e2e-admin-password";
export const ADMIN_EMAIL = "admin@sopho.test";
export const ADMIN_FULL_NAME = "E2E Admin";
export const ENCRYPTION_KEY = "e2e-encryption-key";

export const HEALTH_PATH = "/api/v1/monitor/health";

export const API_ROUTES = {
  SESSION: "/api/v1/auth/session",
  INVITATION_SIGN_UP: "/api/v1/invitation_link/sign_up",
  INVITATIONS: "/invitation",
  TENANTS: "/api/v1/tenant",
  TENANT_SCOPE: "/api/v1/tenant/:tenantId",
  CONNECTIONS: "/connection",
  CANVASES: "/canvas",
  NOTEBOOKS_BY_CANVAS: "/notebook/canvas/:canvasId",
  DASHBOARD_BY_CANVAS: "/dashboard/canvas/:canvasId",
  DASHBOARD: "/dashboard/:dashboardId",
  CELLS: "/cell",
  AI_CONFIGURATION: "/ai_configuration",
  CANVAS: "/canvas/:canvasId",
  EXECUTE_CELL: "/cell/:cellId/execute",
} as const;

export const DEMO_CONNECTION_NAME = "Ecommerce Sales Demo";

export const POSTGRES_IMAGE = "postgres:17";
export const SAMPLE_POSTGRES_DATABASE = "sample";
export const SAMPLE_POSTGRES_USERNAME = "sample_reader";
export const SAMPLE_POSTGRES_PASSWORD = "sample-reader-password";
export const SAMPLE_POSTGRES_SCHEMA = "public";
export const SAMPLE_DATA_PATH = path.join(
  REPOSITORY_DIRECTORY,
  "backend",
  "tests",
  "api",
  "fixtures",
  "sample_data.sql",
);
export const POSTGRES_INIT_SCRIPT_PATH =
  "/docker-entrypoint-initdb.d/sample_data.sql";
export const SAMPLE_POSTGRES_HOST_VARIABLE = "E2E_SAMPLE_POSTGRES_HOST";
export const SAMPLE_POSTGRES_PORT_VARIABLE = "E2E_SAMPLE_POSTGRES_PORT";

export const ORDER_STATUS_QUESTION = "How many orders are in each status?";
export const ORDER_STATUS_SQL =
  "SELECT order_status, COUNT(*) AS order_count FROM orders GROUP BY order_status ORDER BY order_count DESC, order_status";
export const ORDER_STATUS_NARRATION =
  "Delivered orders far outnumber every other status.";
export const DASHBOARD_SUMMARY =
  "Most orders on this dashboard were delivered.";
export const ORDER_COUNT_SQL = "SELECT COUNT(*) AS order_count FROM orders";
export const DEMO_ORDER_COUNT = "10000";

export const SOPHO_IMAGE_VARIABLE = "SOPHO_IMAGE";
export const SOPHO_UPGRADE_FROM_IMAGE_VARIABLE = "SOPHO_UPGRADE_FROM_IMAGE";
export const UPGRADE_SOURCE_NONE = "none";
export const DATABASE_RESET_RELEASE_REASON = "database reset release";
export const SOPHO_CONTAINER_PORT = 8000;
export const SOPHO_CONTAINER_DATA_DIRECTORY = "/app/backend/data";
export const SOPHO_CONTAINER_STARTUP_TIMEOUT_MILLISECONDS = 120_000;
export const METADATA_POSTGRES_ALIAS = "metadata-postgres";
export const METADATA_POSTGRES_DATABASE = "sopho";
export const METADATA_POSTGRES_USERNAME = "sopho";
export const METADATA_POSTGRES_PASSWORD = "sopho-metadata-password";
export const METADATA_POSTGRES_URL = `postgresql://${METADATA_POSTGRES_USERNAME}:${METADATA_POSTGRES_PASSWORD}@${METADATA_POSTGRES_ALIAS}:5432/${METADATA_POSTGRES_DATABASE}`;
export const CONTAINER_LOGS_ATTACHMENT_NAME = "sopho container logs";
