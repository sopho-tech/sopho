import { DEMO_CONNECTION_NAME } from "e2e/support/constants";
import { expect, test } from "e2e/support/fixtures";
import { APP_URLS } from "e2e/support/navigation";
import { samplePostgresDatabase } from "e2e/support/sample_postgres";

test(
  "opens the seeded demo connection",
  { tag: "@smoke" },
  async ({ signedInPage: page }) => {
    await page.goto(APP_URLS.CONNECTIONS);
    const demoConnectionRow = page.getByRole("row", {
      name: new RegExp(DEMO_CONNECTION_NAME),
    });
    await expect(demoConnectionRow).toContainText("ACTIVE");

    await demoConnectionRow.getByRole("button").click();
    await page.getByRole("menuitem", { name: "View" }).click();

    await expect(
      page.getByRole("textbox", { disabled: true }).first(),
    ).toHaveValue(DEMO_CONNECTION_NAME);
  },
);

test("adds a PostgreSQL connection", async ({ signedInPage: page }) => {
  const samplePostgres = samplePostgresDatabase();
  const connectionName = `Sample warehouse ${Date.now()}`;
  await page.goto(APP_URLS.NEW_CONNECTION);

  await page.getByPlaceholder("Connection name").fill(connectionName);
  await page.getByRole("combobox").click();
  await page.getByRole("option", { name: "Postgresql" }).click();
  await page.getByPlaceholder("admin").fill(samplePostgres.username);
  await page
    .getByPlaceholder("toughpassword@123")
    .fill(samplePostgres.password);
  await page.getByPlaceholder("localhost").fill(samplePostgres.host);
  await page.getByPlaceholder("5432").fill(samplePostgres.port);
  await page.getByPlaceholder("my_database").fill(samplePostgres.database);
  await page.getByPlaceholder("public").fill(samplePostgres.schema);
  await page.getByRole("button", { name: "Save" }).click();

  await expect(page).toHaveURL(APP_URLS.CONNECTIONS);
  await expect(
    page.getByRole("row", { name: new RegExp(connectionName) }),
  ).toContainText("ACTIVE");
});
