import { test as base, type Page } from "@playwright/test";
import { signIn } from "e2e/support/api";

type SophoFixtures = {
  signedInPage: Page;
};

export const test = base.extend<SophoFixtures>({
  signedInPage: async ({ page }, use) => {
    await signIn(page.request);
    await use(page);
  },
});

export { expect } from "@playwright/test";
