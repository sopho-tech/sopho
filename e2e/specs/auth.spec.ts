import { ADMIN_EMAIL, ADMIN_PASSWORD, API_ROUTES } from "e2e/support/constants";
import { expect, test } from "e2e/support/fixtures";
import {
  APP_URLS,
  TENANT_HOME_URL_PATTERN,
  tenantUrl,
} from "e2e/support/navigation";

test(
  "signs in with the admin account and signs out",
  { tag: "@smoke" },
  async ({ page }) => {
    await page.goto(APP_URLS.SIGN_IN);
    await page.getByPlaceholder("email").fill(ADMIN_EMAIL);
    await page.getByPlaceholder("password").fill(ADMIN_PASSWORD);
    await page.getByRole("button", { name: "Sign In" }).click();
    await expect(page).toHaveURL(TENANT_HOME_URL_PATTERN);
    const tenantId = TENANT_HOME_URL_PATTERN.exec(new URL(page.url()).pathname)
      ?.groups?.tenantId;
    if (!tenantId) {
      throw new Error(`no workspace id in ${page.url()}`);
    }

    await page.goto(tenantUrl(tenantId, APP_URLS.PROFILE));
    await page.getByRole("button", { name: "Sign Out" }).click();
    await expect(page).toHaveURL(APP_URLS.SIGN_IN);

    await page.goto(tenantUrl(tenantId, APP_URLS.CONNECTIONS));
    await expect(page).toHaveURL(APP_URLS.SIGN_IN);
  },
);

test("keeps a visitor with a wrong password on the sign-in page", async ({
  page,
}) => {
  await page.goto(APP_URLS.SIGN_IN);
  await page.getByPlaceholder("email").fill(ADMIN_EMAIL);
  await page.getByPlaceholder("password").fill("not-the-admin-password");
  const signInResponse = page.waitForResponse(
    (response) =>
      response.url().endsWith(API_ROUTES.SESSION) &&
      response.request().method() === "POST",
  );
  await page.getByRole("button", { name: "Sign In" }).click();

  expect((await signInResponse).status()).toBe(401);
  await expect(page).toHaveURL(APP_URLS.SIGN_IN);
});
