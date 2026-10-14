import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { Canvas } from "src/components/Canvases/Canvas";
import { Dashboard } from "src/components/Dashboard";
import { APP_ROUTES } from "src/constants/app_routes";
import { canvasPageHandlers } from "src/test/msw/handlers";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  CANVAS_ID,
  CHART_CELL,
  ORDER_STATUS_CHART_RESULT,
  ORDER_STATUS_RESULT,
  SQL_CELL,
  dashboardWithCharts,
} from "src/test/test_data";

describe("dashboard", () => {
  test("lays out each chart of the dashboard with its series", async () => {
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
      routes: [
        {
          path: APP_ROUTES.CANVAS,
          element: <Canvas />,
          children: [
            {
              path: APP_ROUTES.CANVAS_ROUTES.DASHBOARD,
              element: <Dashboard />,
            },
          ],
        },
      ],
      initialPath: `${APP_ROUTES.CANVAS.replace(":id", CANVAS_ID)}/${APP_ROUTES.CANVAS_ROUTES.DASHBOARD}`,
    });

    await expect
      .element(page.getByRole("heading", { name: CHART_CELL.name ?? "" }))
      .toBeVisible();
    await expect
      .element(page.getByText("Orders", { exact: true }))
      .toBeVisible();
    await expect
      .element(page.getByText("Largest status", { exact: true }))
      .toBeVisible();
    await expect.element(page.getByText("delivered")).toBeVisible();
  });
});
