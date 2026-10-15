import { defineConfig, devices } from "@playwright/test";
import {
  SOPHO_IMAGE_VARIABLE,
  SOPHO_UPGRADE_FROM_IMAGE_VARIABLE,
} from "e2e/support/constants";
import type { SophoTarget } from "e2e/support/fixtures";
import { requiredEnvironmentVariable } from "e2e/support/sopho_image";

requiredEnvironmentVariable(SOPHO_IMAGE_VARIABLE);
requiredEnvironmentVariable(SOPHO_UPGRADE_FROM_IMAGE_VARIABLE);

const isContinuousIntegration = Boolean(process.env.CI);

export default defineConfig<{ sophoTarget: SophoTarget }>({
  fullyParallel: false,
  workers: 1,
  forbidOnly: isContinuousIntegration,
  retries: isContinuousIntegration ? 2 : 0,
  timeout: 180_000,
  reporter: [
    ["list"],
    ["html", { open: "never" }],
    ["junit", { outputFile: "junit.xml" }],
  ],
  use: {
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "image-sqlite",
      testDir: "specs",
      grep: /@smoke/,
      use: { ...devices["Desktop Chrome"], sophoTarget: "image-sqlite" },
    },
    {
      name: "image-postgres",
      testDir: "specs",
      grep: /@smoke/,
      use: { ...devices["Desktop Chrome"], sophoTarget: "image-postgres" },
    },
    {
      name: "image-upgrade",
      testDir: "upgrade_specs",
    },
  ],
});
