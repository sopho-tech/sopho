import { randomUUID } from "node:crypto";
import { expect, request, test } from "@playwright/test";
import { executeCell, getCanvas, signIn } from "e2e/support/api";
import {
  CONTAINER_LOGS_ATTACHMENT_NAME,
  DATABASE_RESET_RELEASE_REASON,
  DEMO_ORDER_COUNT,
  METADATA_POSTGRES_URL,
  SOPHO_IMAGE_VARIABLE,
  SOPHO_UPGRADE_FROM_IMAGE_VARIABLE,
  UPGRADE_SOURCE_NONE,
} from "e2e/support/constants";
import {
  createDataVolume,
  removeDataVolume,
  requiredEnvironmentVariable,
  startMetadataPostgres,
  startSopho,
  stopMetadataPostgres,
  upgradeEnvironment,
  type SophoContainerOptions,
  type StartedSopho,
} from "e2e/support/sopho_image";
import {
  seedCanvasThroughPreviousRelease,
  type SeededCanvas,
} from "e2e/upgrade_specs/previous_release_api";

const upgradeSourceImage = requiredEnvironmentVariable(
  SOPHO_UPGRADE_FROM_IMAGE_VARIABLE,
);
const imageUnderTest = requiredEnvironmentVariable(SOPHO_IMAGE_VARIABLE);

const startedContainers: { label: string; sopho: StartedSopho }[] = [];
const stoppedContainers = new Set<StartedSopho>();

async function startTrackedSopho(
  label: string,
  options: SophoContainerOptions,
): Promise<StartedSopho> {
  const sopho = await startSopho(options);
  startedContainers.push({ label, sopho });
  return sopho;
}

async function stopTrackedSopho(sopho: StartedSopho): Promise<void> {
  await sopho.container.stop();
  stoppedContainers.add(sopho);
}

async function stopRunningSophoContainers(): Promise<void> {
  for (const { sopho } of startedContainers) {
    if (!stoppedContainers.has(sopho)) {
      await stopTrackedSopho(sopho);
    }
  }
}

async function verifyUpgradedCanvas(
  baseUrl: string,
  seededCanvas: SeededCanvas,
): Promise<void> {
  const upgraded = await request.newContext({ baseURL: baseUrl });
  await signIn(upgraded);
  const canvas = await getCanvas(upgraded, seededCanvas.canvasId);
  expect(canvas.name).toBe(seededCanvas.canvasName);
  const result = await executeCell(upgraded, seededCanvas.sqlCellId);
  expect(result.data).toEqual([{ order_count: Number(DEMO_ORDER_COUNT) }]);
  await upgraded.dispose();
}

test.describe("upgrading from the previous release", () => {
  test.skip(
    upgradeSourceImage === UPGRADE_SOURCE_NONE,
    DATABASE_RESET_RELEASE_REASON,
  );

  test.afterEach(async () => {
    const testInfo = test.info();
    if (testInfo.status !== testInfo.expectedStatus) {
      for (const { label, sopho } of startedContainers) {
        await testInfo.attach(`${CONTAINER_LOGS_ATTACHMENT_NAME} (${label})`, {
          body: sopho.logs.join(""),
          contentType: "text/plain",
        });
      }
    }
    startedContainers.length = 0;
    stoppedContainers.clear();
  });

  test("keeps a canvas and its SQL on SQLite", async () => {
    const dataVolumeName = `sopho-upgrade-${randomUUID()}`;
    await createDataVolume(dataVolumeName);
    try {
      const previousRelease = await startTrackedSopho("previous release", {
        image: upgradeSourceImage,
        environment: upgradeEnvironment(null),
        network: null,
        dataVolumeName,
      });
      const seededCanvas = await seedCanvasThroughPreviousRelease(
        previousRelease.baseUrl,
        "Canvas from the previous release",
      );
      await stopTrackedSopho(previousRelease);

      const upgraded = await startTrackedSopho("image under test", {
        image: imageUnderTest,
        environment: upgradeEnvironment(null),
        network: null,
        dataVolumeName,
      });
      await verifyUpgradedCanvas(upgraded.baseUrl, seededCanvas);
    } finally {
      await stopRunningSophoContainers();
      await removeDataVolume(dataVolumeName);
    }
  });

  test("keeps a canvas and its SQL on Postgres", async () => {
    const metadataPostgres = await startMetadataPostgres();
    try {
      const previousRelease = await startTrackedSopho("previous release", {
        image: upgradeSourceImage,
        environment: upgradeEnvironment(METADATA_POSTGRES_URL),
        network: metadataPostgres.network,
        dataVolumeName: null,
      });
      const seededCanvas = await seedCanvasThroughPreviousRelease(
        previousRelease.baseUrl,
        "Canvas from the previous release",
      );
      await stopTrackedSopho(previousRelease);

      const upgraded = await startTrackedSopho("image under test", {
        image: imageUnderTest,
        environment: upgradeEnvironment(METADATA_POSTGRES_URL),
        network: metadataPostgres.network,
        dataVolumeName: null,
      });
      await verifyUpgradedCanvas(upgraded.baseUrl, seededCanvas);
    } finally {
      await stopRunningSophoContainers();
      await stopMetadataPostgres(metadataPostgres);
    }
  });
});
