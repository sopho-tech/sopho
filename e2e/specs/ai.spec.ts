import {
  configureAnthropicProvider,
  createCanvas,
  createOrderStatusChart,
  createSqlCell,
  findDemoConnection,
  lastMessageEventNames,
  placeChartOnDashboard,
} from "e2e/support/api";
import {
  ANSWER_PERSISTED_TIMEOUT_MILLISECONDS,
  COMPLETED_EVENT_NAME,
  DASHBOARD_SUMMARY,
  DEMO_CONNECTION_NAME,
  FAKE_LLM_API_KEY,
  ORDER_STATUS_NARRATION,
  ORDER_STATUS_QUESTION,
  ORDER_STATUS_SQL,
} from "e2e/support/constants";
import { expect, test } from "e2e/support/fixtures";
import {
  APP_URLS,
  CONVERSATION_URL_PATTERN,
  canvasDashboardUrl,
} from "e2e/support/navigation";

test("configures the Anthropic provider", async ({ signedInPage: page }) => {
  await page.goto(APP_URLS.AI_CONFIGURATIONS);

  await page.getByRole("combobox").click();
  await page.getByRole("option", { name: "Anthropic" }).click();
  await page
    .getByRole("textbox", { name: "Provider API key" })
    .fill(FAKE_LLM_API_KEY);
  await page.getByRole("button", { name: "Save" }).click();

  await expect(
    page.getByText("Anthropic — live (connection verified)"),
  ).toBeVisible();
});

test("answers a question with SQL and a chart", async ({
  signedInPage: page,
}) => {
  await configureAnthropicProvider(page.request);
  await page.goto(APP_URLS.NEW_CONVERSATION);

  await page.getByRole("combobox").click();
  await page.getByRole("option", { name: DEMO_CONNECTION_NAME }).click();
  const composer = page.getByRole("textbox", {
    name: "Ask anything about your data",
  });
  await composer.fill(ORDER_STATUS_QUESTION);
  await composer.press("Enter");
  await expect(page).toHaveURL(CONVERSATION_URL_PATTERN);
  const conversationId = CONVERSATION_URL_PATTERN.exec(
    new URL(page.url()).pathname,
  )?.groups?.conversationId;
  if (!conversationId) {
    throw new Error(`no conversation id in ${page.url()}`);
  }
  await expect
    .poll(() => lastMessageEventNames(page.request, conversationId), {
      timeout: ANSWER_PERSISTED_TIMEOUT_MILLISECONDS,
    })
    .toContain(COMPLETED_EVENT_NAME);
  await page.reload();

  await expect(page.getByText(ORDER_STATUS_NARRATION)).toBeVisible();
  await expect(
    page.getByRole("application").filter({ hasText: "delivered" }),
  ).toBeVisible();
  await expect(
    page.getByText("Which states have the most delivered orders?"),
  ).toBeVisible();
  await page.getByText("Explanation").click();
  await page.getByText("Generated SQL").click();
  await expect(page.getByText(ORDER_STATUS_SQL)).toBeVisible();
});

test("summarises a dashboard", async ({ signedInPage: page }) => {
  await configureAnthropicProvider(page.request);
  const demoConnection = await findDemoConnection(page.request);
  const canvas = await createCanvas(page.request, "Summarised dashboard");
  const sqlCell = await createSqlCell(
    page.request,
    canvas.notebookId,
    demoConnection.id,
    "Orders by status",
    ORDER_STATUS_SQL,
  );
  const chartCell = await createOrderStatusChart(
    page.request,
    canvas.notebookId,
    demoConnection.id,
    sqlCell.id,
  );
  await placeChartOnDashboard(page.request, canvas, chartCell.id);
  await page.goto(canvasDashboardUrl(canvas.canvasId));

  await page
    .getByRole("button", { name: "Summarise dashboard", exact: true })
    .click();
  await page.getByText("AI summary").click();

  await expect(page.getByText(DASHBOARD_SUMMARY)).toBeVisible();
});
