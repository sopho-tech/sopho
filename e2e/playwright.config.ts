import { defineConfig, devices } from "@playwright/test";
import {
  ADMIN_EMAIL,
  ADMIN_FULL_NAME,
  ADMIN_PASSWORD,
  ADMIN_USERNAME,
  DOCKER_SHUTDOWN_TIMEOUT_MILLISECONDS,
  E2E_DATA_DIRECTORY,
  EMAIL_SENDER,
  ENCRYPTION_KEY,
  FAKE_LLM_BASE_URL,
  FAKE_LLM_BINARY_PATH,
  FAKE_LLM_FIXTURES_DIRECTORY,
  FAKE_LLM_READY_MESSAGE,
  FRONTEND_DIST_DIRECTORY,
  HEALTH_PATH,
  MAILPIT_BASE_URL,
  MAILPIT_CONTAINER_NAME,
  MAILPIT_HTTP_PORT,
  MAILPIT_IMAGE,
  MAILPIT_READY_PATH,
  MAILPIT_SMTP_PORT,
  METADATA_DATABASE_URL,
  MOCK_IDENTITY_CONFIGURATION,
  MOCK_IDENTITY_CONTAINER_NAME,
  MOCK_IDENTITY_DISCOVERY_PATH,
  MOCK_IDENTITY_IMAGE,
  MOCK_IDENTITY_ISSUER_URL,
  MOCK_IDENTITY_PORT,
  SINGLE_SIGN_ON_CLIENT_ID,
  SINGLE_SIGN_ON_CLIENT_SECRET,
  SINGLE_SIGN_ON_PROVIDER,
  SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME,
  SINGLE_SIGN_ON_REDIRECT_URI,
  SOPHO_BASE_URL,
  SOPHO_BINARY_PATH,
  SOPHO_PORT,
} from "e2e/support/constants";

const DOCKER_GRACEFUL_SHUTDOWN = {
  signal: "SIGTERM",
  timeout: DOCKER_SHUTDOWN_TIMEOUT_MILLISECONDS,
} as const;

function dockerRunCommand(
  containerName: string,
  dockerArguments: string,
  image: string,
): string {
  return `docker rm --force ${containerName} >/dev/null 2>&1; exec docker run --rm --name ${containerName} ${dockerArguments} ${image}`;
}

const singleSignOnVariablePrefix = `SOPHO_OIDC_${SINGLE_SIGN_ON_PROVIDER.toUpperCase()}`;

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
      name: "mailpit",
      command: dockerRunCommand(
        MAILPIT_CONTAINER_NAME,
        `--publish 127.0.0.1:${MAILPIT_SMTP_PORT}:1025 --publish 127.0.0.1:${MAILPIT_HTTP_PORT}:8025`,
        MAILPIT_IMAGE,
      ),
      url: `${MAILPIT_BASE_URL}${MAILPIT_READY_PATH}`,
      reuseExistingServer: false,
      gracefulShutdown: DOCKER_GRACEFUL_SHUTDOWN,
      stdout: "pipe",
    },
    {
      name: "mock-identity-provider",
      command: dockerRunCommand(
        MOCK_IDENTITY_CONTAINER_NAME,
        `--publish 127.0.0.1:${MOCK_IDENTITY_PORT}:8080 --env JSON_CONFIG='${MOCK_IDENTITY_CONFIGURATION}'`,
        MOCK_IDENTITY_IMAGE,
      ),
      url: `${MOCK_IDENTITY_ISSUER_URL}${MOCK_IDENTITY_DISCOVERY_PATH}`,
      reuseExistingServer: false,
      gracefulShutdown: DOCKER_GRACEFUL_SHUTDOWN,
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
        SOPHO_FRONTEND_DIR: FRONTEND_DIST_DIRECTORY,
        SOPHO_ENVIRONMENT: "production",
        SOPHO_COOKIE_SECURE: "false",
        SOPHO_ENCRYPTION_KEY: ENCRYPTION_KEY,
        SOPHO_ADMIN_USERNAME: ADMIN_USERNAME,
        SOPHO_ADMIN_PASSWORD: ADMIN_PASSWORD,
        SOPHO_ADMIN_EMAIL: ADMIN_EMAIL,
        SOPHO_ADMIN_FULL_NAME: ADMIN_FULL_NAME,
        SOPHO_ANTHROPIC_BASE_URL: FAKE_LLM_BASE_URL,
        SOPHO_OPENAI_BASE_URL: `${FAKE_LLM_BASE_URL}/v1`,
        SOPHO_SMTP_HOST: "127.0.0.1",
        SOPHO_SMTP_PORT: String(MAILPIT_SMTP_PORT),
        SOPHO_SMTP_TLS: "none",
        SOPHO_EMAIL_FROM: EMAIL_SENDER,
        SOPHO_PUBLIC_URL: SOPHO_BASE_URL,
        SOPHO_OIDC_PROVIDERS: SINGLE_SIGN_ON_PROVIDER,
        [`${singleSignOnVariablePrefix}_DISPLAY_NAME`]:
          SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME,
        [`${singleSignOnVariablePrefix}_ISSUER_URL`]: MOCK_IDENTITY_ISSUER_URL,
        [`${singleSignOnVariablePrefix}_CLIENT_ID`]: SINGLE_SIGN_ON_CLIENT_ID,
        [`${singleSignOnVariablePrefix}_CLIENT_SECRET`]:
          SINGLE_SIGN_ON_CLIENT_SECRET,
        [`${singleSignOnVariablePrefix}_REDIRECT_URI`]:
          SINGLE_SIGN_ON_REDIRECT_URI,
      },
    },
  ],
});
