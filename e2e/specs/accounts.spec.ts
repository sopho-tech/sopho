import { randomUUID } from "node:crypto";
import type {
  APIRequestContext,
  Browser,
  BrowserContext,
  Page,
} from "@playwright/test";
import { inviteToTenant, signUpFromInvitation } from "e2e/support/api";
import {
  API_ROUTES,
  INVITATION_LINK_PATH,
  INVITATION_SUBJECT,
  INVITED_ROLE,
  INVITEE_EMAIL_DOMAIN,
  INVITEE_PASSWORD,
  MOCK_IDENTITY_CLAIMS_FIELD,
  MOCK_IDENTITY_SUBJECT_FIELD,
  MOCK_IDENTITY_SUBMIT_LABEL,
  PASSWORD_RESET_SUBJECT,
  RESET_LINK_PATH,
  RESET_PASSWORD,
  RESET_REQUESTED_MESSAGE,
  SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME,
} from "e2e/support/constants";
import { expect, test } from "e2e/support/fixtures";
import { linkInEmail, tokenInLink, waitForEmail } from "e2e/support/mailbox";
import { APP_URLS, tenantUrl } from "e2e/support/navigation";

type Person = {
  email: string;
  fullName: string;
};

function newPerson(label: string): Person {
  return {
    email: `${label}-${randomUUID().slice(0, 8)}@${INVITEE_EMAIL_DOMAIN}`,
    fullName: `${label} person`,
  };
}

function tenantHomeUrl(tenantId: string): RegExp {
  return new RegExp(`/t/${tenantId}/?$`);
}

async function openVisitorPage(
  browser: Browser,
  baseURL: string | undefined,
): Promise<{ context: BrowserContext; page: Page }> {
  const context = await browser.newContext({ baseURL });
  return { context, page: await context.newPage() };
}

async function createPasswordAccount(
  adminRequest: APIRequestContext,
  visitorRequest: APIRequestContext,
  tenantId: string,
  person: Person,
): Promise<void> {
  await inviteToTenant(adminRequest, tenantId, person.email, INVITED_ROLE);
  const invitation = await waitForEmail(
    adminRequest,
    person.email,
    INVITATION_SUBJECT,
  );
  await signUpFromInvitation(
    visitorRequest,
    tokenInLink(linkInEmail(invitation, INVITATION_LINK_PATH)),
    person.fullName,
    INVITEE_PASSWORD,
  );
}

async function signInWithPassword(
  page: Page,
  email: string,
  password: string,
): Promise<void> {
  await page.goto(APP_URLS.SIGN_IN);
  await page.getByLabel("Email", { exact: true }).fill(email);
  await page.getByLabel("Password", { exact: true }).fill(password);
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
}

async function signInAtIdentityProvider(
  page: Page,
  subject: string,
  person: Person,
): Promise<void> {
  await page
    .locator(`input[name="${MOCK_IDENTITY_SUBJECT_FIELD}"]`)
    .fill(subject);
  await page.locator(`textarea[name="${MOCK_IDENTITY_CLAIMS_FIELD}"]`).fill(
    JSON.stringify({
      email: person.email,
      email_verified: true,
      name: person.fullName,
    }),
  );
  await page.getByRole("button", { name: MOCK_IDENTITY_SUBMIT_LABEL }).click();
}

test("a new person joins from the emailed link", async ({
  signedInPage,
  tenantId,
  browser,
  baseURL,
}) => {
  const newcomer = newPerson("newcomer");
  await signedInPage.goto(tenantUrl(tenantId, APP_URLS.MEMBERS));
  await signedInPage.getByPlaceholder("Email address").fill(newcomer.email);
  await signedInPage
    .getByRole("combobox")
    .filter({ hasText: /^Role$/ })
    .click();
  await signedInPage
    .getByRole("option", { name: INVITED_ROLE, exact: true })
    .click();
  await signedInPage
    .getByRole("button", { name: "Invite", exact: true })
    .click();
  await expect(
    signedInPage.getByText(
      `Invited ${newcomer.email}. Sopho is emailing them a link to join.`,
    ),
  ).toBeVisible();

  const invitation = await waitForEmail(
    signedInPage.request,
    newcomer.email,
    INVITATION_SUBJECT,
  );
  const visitor = await openVisitorPage(browser, baseURL);
  await visitor.page.goto(linkInEmail(invitation, INVITATION_LINK_PATH));
  await visitor.page.getByPlaceholder("Full name").fill(newcomer.fullName);
  await visitor.page
    .getByPlaceholder("Choose a password")
    .fill(INVITEE_PASSWORD);
  await visitor.page.getByRole("button", { name: "Create account" }).click();
  await expect(visitor.page).toHaveURL(tenantHomeUrl(tenantId));
  await visitor.context.close();

  await signedInPage.reload();
  await expect(
    signedInPage.getByRole("listitem", { name: newcomer.email }),
  ).toContainText(INVITED_ROLE);
});

test("forgot password emails a link that sets a new password", async ({
  signedInPage,
  tenantId,
  browser,
  baseURL,
}) => {
  const forgetful = newPerson("forgetful");
  const visitor = await openVisitorPage(browser, baseURL);
  await createPasswordAccount(
    signedInPage.request,
    visitor.page.request,
    tenantId,
    forgetful,
  );
  await visitor.context.clearCookies();

  await visitor.page.goto(APP_URLS.SIGN_IN);
  await visitor.page.getByRole("button", { name: "Forgot password?" }).click();
  await visitor.page.getByPlaceholder("email").fill(forgetful.email);
  await visitor.page.getByRole("button", { name: "Send reset link" }).click();
  await expect(visitor.page.getByText(RESET_REQUESTED_MESSAGE)).toBeVisible();

  const resetEmail = await waitForEmail(
    signedInPage.request,
    forgetful.email,
    PASSWORD_RESET_SUBJECT,
  );
  await visitor.page.goto(linkInEmail(resetEmail, RESET_LINK_PATH));
  await visitor.page.getByPlaceholder("New password").fill(RESET_PASSWORD);
  await visitor.page
    .getByRole("button", { name: "Choose new password" })
    .click();
  await visitor.page.getByRole("button", { name: "Sign in" }).click();
  await expect(visitor.page).toHaveURL(APP_URLS.SIGN_IN);

  const rejectedSignIn = visitor.page.waitForResponse(
    (response) =>
      response.url().endsWith(API_ROUTES.SESSION) &&
      response.request().method() === "POST",
  );
  await signInWithPassword(visitor.page, forgetful.email, INVITEE_PASSWORD);
  expect((await rejectedSignIn).status()).toBe(401);
  await signInWithPassword(visitor.page, forgetful.email, RESET_PASSWORD);
  await expect(visitor.page).toHaveURL(tenantHomeUrl(tenantId));
  await visitor.context.close();
});

test("connecting single sign-on from the profile, then signing in with it", async ({
  signedInPage,
  tenantId,
  browser,
  baseURL,
}) => {
  const connector = newPerson("connector");
  const subject = `connector-subject-${randomUUID()}`;
  const visitor = await openVisitorPage(browser, baseURL);
  await createPasswordAccount(
    signedInPage.request,
    visitor.page.request,
    tenantId,
    connector,
  );

  await visitor.page.goto(APP_URLS.PROFILE);
  await visitor.page
    .getByRole("listitem", { name: SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME })
    .getByRole("button", { name: "Connect" })
    .click();
  await signInAtIdentityProvider(visitor.page, subject, connector);
  await expect(
    visitor.page.getByText(
      `${SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME} is connected. You can now sign in with it.`,
    ),
  ).toBeVisible();
  await expect(
    visitor.page.getByRole("listitem", {
      name: SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME,
    }),
  ).toContainText(`Connected as ${connector.email}`);

  await visitor.page.getByRole("button", { name: "Sign Out" }).click();
  await expect(visitor.page).toHaveURL(APP_URLS.SIGN_IN);
  await visitor.page
    .getByRole("button", {
      name: SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME,
      exact: true,
    })
    .click();
  await signInAtIdentityProvider(visitor.page, subject, connector);
  await expect(visitor.page).toHaveURL(tenantHomeUrl(tenantId));
  await visitor.context.close();
});
