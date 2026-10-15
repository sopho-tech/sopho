import { test as base, type Page } from "@playwright/test";
import { signIn } from "e2e/support/api";
import {
  CONTAINER_LOGS_ATTACHMENT_NAME,
  METADATA_POSTGRES_URL,
  SOPHO_BASE_URL,
  SOPHO_IMAGE_VARIABLE,
} from "e2e/support/constants";
import {
  readmeEnvironment,
  requiredEnvironmentVariable,
  startMetadataPostgres,
  startSopho,
  stopMetadataPostgres,
} from "e2e/support/sopho_image";

export type SophoTarget = "native" | "image-sqlite" | "image-postgres";

type SophoServer = {
  baseUrl: string;
  logs: string[] | null;
};

type SophoTestFixtures = {
  signedInPage: Page;
  attachContainerLogsOnFailure: void;
};

type SophoWorkerFixtures = {
  sophoTarget: SophoTarget;
  sophoServer: SophoServer;
};

export const test = base.extend<SophoTestFixtures, SophoWorkerFixtures>({
  sophoTarget: ["native", { scope: "worker", option: true }],
  sophoServer: [
    async ({ sophoTarget }, use) => {
      switch (sophoTarget) {
        case "native": {
          await use({ baseUrl: SOPHO_BASE_URL, logs: null });
          return;
        }
        case "image-sqlite": {
          const sopho = await startSopho({
            image: requiredEnvironmentVariable(SOPHO_IMAGE_VARIABLE),
            environment: readmeEnvironment(),
            network: null,
            dataVolumeName: null,
          });
          await use({ baseUrl: sopho.baseUrl, logs: sopho.logs });
          await sopho.container.stop();
          return;
        }
        case "image-postgres": {
          const metadataPostgres = await startMetadataPostgres();
          const sopho = await startSopho({
            image: requiredEnvironmentVariable(SOPHO_IMAGE_VARIABLE),
            environment: {
              ...readmeEnvironment(),
              DATABASE_URL: METADATA_POSTGRES_URL,
            },
            network: metadataPostgres.network,
            dataVolumeName: null,
          });
          await use({ baseUrl: sopho.baseUrl, logs: sopho.logs });
          await sopho.container.stop();
          await stopMetadataPostgres(metadataPostgres);
          return;
        }
        default: {
          const unknownTarget: never = sophoTarget;
          throw new Error(`unknown sopho target ${String(unknownTarget)}`);
        }
      }
    },
    { scope: "worker" },
  ],
  baseURL: async ({ sophoServer }, use) => {
    await use(sophoServer.baseUrl);
  },
  attachContainerLogsOnFailure: [
    async ({ sophoServer }, use, testInfo) => {
      await use();
      if (
        sophoServer.logs !== null &&
        testInfo.status !== testInfo.expectedStatus
      ) {
        await testInfo.attach(CONTAINER_LOGS_ATTACHMENT_NAME, {
          body: sophoServer.logs.join(""),
          contentType: "text/plain",
        });
      }
    },
    { auto: true },
  ],
  signedInPage: async ({ page }, use) => {
    await signIn(page.request);
    await use(page);
  },
});

export { expect } from "@playwright/test";
