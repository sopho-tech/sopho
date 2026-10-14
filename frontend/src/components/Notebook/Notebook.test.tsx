import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page, userEvent } from "vitest/browser";
import { Canvas } from "src/components/Canvases/Canvas";
import { Notebook } from "src/components/Notebook";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { canvasPageHandlers } from "src/test/msw/handlers";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  CANVAS_ID,
  CHART_CELL,
  ORDER_COUNT_RESULT,
  ORDER_COUNT_SQL,
  ORDER_STATUS_CHART_RESULT,
  ORDER_STATUS_RESULT,
  SQL_CELL,
  dashboardWithCharts,
} from "src/test/test_data";

const notebookRoutes = [
  {
    path: APP_ROUTES.CANVAS,
    element: <Canvas />,
    children: [
      { path: APP_ROUTES.CANVAS_ROUTES.NOTEBOOK, element: <Notebook /> },
    ],
  },
];

const notebookPath = `${APP_ROUTES.CANVAS.replace(":id", CANVAS_ID)}/${APP_ROUTES.CANVAS_ROUTES.NOTEBOOK}`;

async function runActiveCell() {
  await userEvent.keyboard("{Meta>}{Enter}{/Meta}");
}

describe("notebook", () => {
  test("running a SQL cell shows every row with its row number", async () => {
    worker.use(
      ...canvasPageHandlers({
        cells: [SQL_CELL],
        dashboard: dashboardWithCharts([]),
        executionResults: { [SQL_CELL.id]: ORDER_STATUS_RESULT },
      }),
    );
    await renderWithProviders({
      routes: notebookRoutes,
      initialPath: notebookPath,
    });

    await page.getByText(`${ORDER_COUNT_SQL};`).click();
    await runActiveCell();

    await expect
      .element(page.getByRole("row", { name: "order_status order_count" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("row", { name: "1 delivered 9620" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("row", { name: "2 shipped 118" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("row", { name: "3 canceled 75" }))
      .toBeVisible();
  });

  test("running a selection executes only the selected SQL", async () => {
    let previewRequest: unknown = null;
    worker.use(
      ...canvasPageHandlers({
        cells: [SQL_CELL],
        dashboard: dashboardWithCharts([]),
        executionResults: {},
      }),
      http.post(API_ENDPOINTS.CELL.EXECUTE_PREVIEW, async ({ request }) => {
        previewRequest = await request.json();
        return HttpResponse.json(ORDER_COUNT_RESULT);
      }),
    );
    await renderWithProviders({
      routes: notebookRoutes,
      initialPath: notebookPath,
    });

    await page.getByText(`${ORDER_COUNT_SQL};`).click();
    await userEvent.keyboard("{Home}{Shift>}{End}{/Shift}");
    await runActiveCell();

    await expect
      .element(page.getByRole("row", { name: "1 10000" }))
      .toBeVisible();
    expect(previewRequest).toEqual({
      content: `${ORDER_COUNT_SQL};`,
      cell_type: "SQL",
    });
  });

  test("the chart cell editor shows each series of the chart", async () => {
    worker.use(
      ...canvasPageHandlers({
        cells: [SQL_CELL, CHART_CELL],
        dashboard: dashboardWithCharts([CHART_CELL]),
        executionResults: {
          [SQL_CELL.id]: ORDER_STATUS_RESULT,
          [CHART_CELL.id]: ORDER_STATUS_CHART_RESULT,
        },
      }),
    );
    await renderWithProviders({
      routes: notebookRoutes,
      initialPath: notebookPath,
    });

    await page.getByText("Y-axis").click();

    await expect
      .element(page.getByRole("heading", { name: "Series 1" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("heading", { name: "Series 2" }))
      .toBeVisible();
    const seriesLabels = page.getByPlaceholder("legend label");
    await expect.element(seriesLabels.nth(0)).toHaveValue("Orders");
    await expect.element(seriesLabels.nth(1)).toHaveValue("Largest status");
  });
});
