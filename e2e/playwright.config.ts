import { defineConfig, devices } from "@playwright/test";
import {
  ADMIN_EMAIL,
  ADMIN_FULL_NAME,
  ADMIN_PASSWORD,
  ADMIN_USERNAME,
  E2E_DATA_DIRECTORY,
  ENCRYPTION_KEY,
  FAKE_LLM_BASE_URL,
  FAKE_LLM_BINARY_PATH,
  FAKE_LLM_FIXTURES_DIRECTORY,
  FAKE_LLM_READY_MESSAGE,
  FRONTEND_DIST_DIRECTORY,
  HEALTH_PATH,
  METADATA_DATABASE_URL,
  SOPHO_BASE_URL,
  SOPHO_BINARY_PATH,
  SOPHO_PORT,
} from "e2e/support/constants";

const isContinuousIntegration = Boolean(process.env.CI);

export default defineConfig({
  testDir: "specs",
  fullyParallel: false,
  workers: 1,
  forbidOnly: isContinuousIntegration,
  retries: isContinuousIntegration ? 2 : 0,
  reporter: [
    ["list"],
    ["html", { open: "never" }],
    ["junit", { outputFile: "junit.xml" }],
  ],
  globalSetup: "./global_setup.ts",
  use: {
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "native",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: [
    {
      name: "fake-llm",
      command: `"${FAKE_LLM_BINARY_PATH}" --port ${new URL(FAKE_LLM_BASE_URL).port} --fixtures "${FAKE_LLM_FIXTURES_DIRECTORY}"`,
      wait: { stdout: FAKE_LLM_READY_MESSAGE },
      reuseExistingServer: false,
      stdout: "pipe",
    },
    {
      name: "sopho",
      command: `rm -rf "${E2E_DATA_DIRECTORY}" && mkdir -p "${E2E_DATA_DIRECTORY}" && exec "${SOPHO_BINARY_PATH}"`,
      url: `${SOPHO_BASE_URL}${HEALTH_PATH}`,
      reuseExistingServer: false,
      env: {
        DATABASE_URL: METADATA_DATABASE_URL,
        PORT: String(SOPHO_PORT),
        FRONTEND_DIR: FRONTEND_DIST_DIRECTORY,
        ENVIRONMENT: "production",
        COOKIE_SECURE: "false",
        ENCRYPTION_KEY,
        ADMIN_USERNAME,
        ADMIN_PASSWORD,
        ADMIN_EMAIL,
        ADMIN_FULL_NAME,
        ANTHROPIC_BASE_URL: FAKE_LLM_BASE_URL,
        OPENAI_BASE_URL: `${FAKE_LLM_BASE_URL}/v1`,
      },
    },
  ],
});
