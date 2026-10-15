import {
  createCanvas,
  createSqlCell,
  findDemoConnection,
} from "e2e/support/api";
import {
  DEMO_ORDER_COUNT,
  ORDER_COUNT_SQL,
  ORDER_STATUS_SQL,
} from "e2e/support/constants";
import { expect, test } from "e2e/support/fixtures";
import { canvasNotebookUrl } from "e2e/support/navigation";

test(
  "runs SQL against the demo data",
  { tag: "@smoke" },
  async ({ signedInPage: page }) => {
    const demoConnection = await findDemoConnection(page.request);
    const canvas = await createCanvas(page.request, "Notebook smoke test");
    await createSqlCell(
      page.request,
      canvas.notebookId,
      demoConnection.id,
      "Order count",
      ORDER_COUNT_SQL,
    );
    await page.goto(canvasNotebookUrl(canvas.canvasId));

    await page.getByText(ORDER_COUNT_SQL).click();
    await page.keyboard.press("Meta+Enter");

    await expect(
      page.getByRole("row", { name: `1 ${DEMO_ORDER_COUNT}` }),
    ).toBeVisible();
  },
);

test("runs only the selected SQL", async ({ signedInPage: page }) => {
  const demoConnection = await findDemoConnection(page.request);
  const canvas = await createCanvas(page.request, "Selected SQL");
  await createSqlCell(
    page.request,
    canvas.notebookId,
    demoConnection.id,
    "Two statements",
    `${ORDER_STATUS_SQL};\n${ORDER_COUNT_SQL};`,
  );
  await page.goto(canvasNotebookUrl(canvas.canvasId));

  await page.getByText(`${ORDER_COUNT_SQL};`).click();
  await page.keyboard.press("Home");
  await page.keyboard.press("Shift+End");
  await page.keyboard.press("Meta+Enter");

  await expect(
    page.getByRole("row", { name: `1 ${DEMO_ORDER_COUNT}` }),
  ).toBeVisible();
  await expect(page.getByRole("row", { name: /delivered/ })).toHaveCount(0);
});
