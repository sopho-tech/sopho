import {
  createCanvas,
  createOrderStatusChart,
  createSqlCell,
  findDemoConnection,
  placeChartOnDashboard,
  type CreatedCanvas,
} from "e2e/support/api";
import { ORDER_STATUS_SQL } from "e2e/support/constants";
import { expect, test } from "e2e/support/fixtures";
import { canvasDashboardUrl, canvasNotebookUrl } from "e2e/support/navigation";
import type { APIRequestContext } from "@playwright/test";

async function createOrderStatusCanvas(
  request: APIRequestContext,
  canvasName: string,
): Promise<{ canvas: CreatedCanvas; chartCellId: string }> {
  const demoConnection = await findDemoConnection(request);
  const canvas = await createCanvas(request, canvasName);
  const sqlCell = await createSqlCell(
    request,
    canvas.notebookId,
    demoConnection.id,
    "Orders by status",
    ORDER_STATUS_SQL,
  );
  const chartCell = await createOrderStatusChart(
    request,
    canvas.notebookId,
    demoConnection.id,
    sqlCell.id,
  );
  return { canvas, chartCellId: chartCell.id };
}

test(
  "renders a chart cell",
  { tag: "@smoke" },
  async ({ signedInPage: page }) => {
    const { canvas } = await createOrderStatusCanvas(
      page.request,
      "Chart smoke test",
    );

    await page.goto(canvasNotebookUrl(canvas.canvasId));

    await expect(page.getByText("Orders", { exact: true })).toBeVisible();
    await expect(
      page.getByRole("application").filter({ hasText: "delivered" }),
    ).toBeVisible();
  },
);

test("shows the chart on the dashboard", async ({ signedInPage: page }) => {
  const { canvas, chartCellId } = await createOrderStatusCanvas(
    page.request,
    "Dashboard chart",
  );
  await placeChartOnDashboard(page.request, canvas, chartCellId);

  await page.goto(canvasDashboardUrl(canvas.canvasId));

  await expect(
    page.getByRole("heading", { name: "Orders by status chart" }),
  ).toBeVisible();
  await expect(
    page.getByRole("application").filter({ hasText: "delivered" }),
  ).toBeVisible();
});
