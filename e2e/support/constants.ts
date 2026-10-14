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

export const ADMIN_USERNAME = "admin";
export const ADMIN_PASSWORD = "e2e-admin-password";
export const ADMIN_EMAIL = "admin@sopho.test";
export const ADMIN_FULL_NAME = "E2E Admin";
export const ENCRYPTION_KEY = "e2e-encryption-key";

export const HEALTH_PATH = "/api/v1/monitor/health";

export const API_ROUTES = {
  SESSION: "/api/v1/auth/session",
  CONNECTIONS: "/api/v1/connection",
  CANVASES: "/api/v1/canvas",
  NOTEBOOKS_BY_CANVAS: "/api/v1/notebook/canvas/:canvasId",
  DASHBOARD_BY_CANVAS: "/api/v1/dashboard/canvas/:canvasId",
  DASHBOARD: "/api/v1/dashboard/:dashboardId",
  CELLS: "/api/v1/cell",
  AI_CONFIGURATION: "/api/v1/ai_configuration",
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
