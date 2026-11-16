import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { Canvas } from "src/components/Canvases/Canvas";
import { Dashboard } from "src/components/Dashboard";
import { APP_ROUTES } from "src/constants/app_routes";
import { canvasPageHandlers } from "src/test/msw/handlers";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  tenantRoutePattern,
  testTenantApiUrl,
  testTenantPath,
} from "src/test/tenant_routes";
import {
  AI_ON_OWN_KEY,
  AI_WITH_THE_ALLOWANCE_USED,
  CANVAS_ID,
  CHART_CELL,
  ORDER_STATUS_CHART_RESULT,
  ORDER_STATUS_RESULT,
  SQL_CELL,
  dashboardWithCharts,
} from "src/test/test_data";

function dashboardWithOneChartHandlers() {
  return canvasPageHandlers({
    cells: [SQL_CELL, CHART_CELL],
    dashboard: dashboardWithCharts([CHART_CELL]),
    executionResults: {
      [SQL_CELL.id]: ORDER_STATUS_RESULT,
      [CHART_CELL.id]: ORDER_STATUS_CHART_RESULT,
    },
  });
}

function renderTheDashboardPage() {
  return renderWithProviders({
    routes: [
      {
        path: tenantRoutePattern(APP_ROUTES.CANVAS),
        element: <Canvas />,
        children: [
          {
            path: APP_ROUTES.CANVAS_ROUTES.DASHBOARD,
            element: <Dashboard />,
          },
        ],
      },
    ],
    initialPath: testTenantPath(
      `${APP_ROUTES.CANVAS.replace(":id", CANVAS_ID)}/${APP_ROUTES.CANVAS_ROUTES.DASHBOARD}`,
    ),
  });
}

const SUMMARISE_CONTROL_NAMES = [
  "Summarise all charts",
  "Summarise dashboard",
  "Summarise chart",
];

describe("dashboard", () => {
  test("lays out each chart of the dashboard with its series", async () => {
    worker.use(...dashboardWithOneChartHandlers());

    await renderTheDashboardPage();

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

  test.for(SUMMARISE_CONTROL_NAMES)(
    "%s is disabled and explains why when the backend says AI cannot be used",
    async (summariseControlName) => {
      worker.use(
        http.get(testTenantApiUrl(API_ENDPOINTS.AI_CONFIGURATION.GET), () =>
          HttpResponse.json(AI_WITH_THE_ALLOWANCE_USED),
        ),
        ...dashboardWithOneChartHandlers(),
      );

      await renderTheDashboardPage();

      const summariseControl = page.getByRole("button", {
        name: summariseControlName,
        exact: true,
      });
      await expect
        .element(summariseControl)
        .toHaveAttribute("aria-disabled", "true");
      await summariseControl.hover();
      await expect
        .element(
          page
            .getByText(AI_WITH_THE_ALLOWANCE_USED.unavailable_message ?? "")
            .first(),
        )
        .toBeVisible();
    },
  );

  test("the summarise controls can be used when the backend says AI is available", async () => {
    worker.use(
      http.get(testTenantApiUrl(API_ENDPOINTS.AI_CONFIGURATION.GET), () =>
        HttpResponse.json(AI_ON_OWN_KEY),
      ),
      ...dashboardWithOneChartHandlers(),
    );

    await renderTheDashboardPage();

    for (const summariseControlName of SUMMARISE_CONTROL_NAMES) {
      await expect
        .element(
          page.getByRole("button", { name: summariseControlName, exact: true }),
        )
        .toBeEnabled();
    }
  });
});
