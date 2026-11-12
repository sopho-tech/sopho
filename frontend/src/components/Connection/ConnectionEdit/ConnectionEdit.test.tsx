import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { ConnectionEdit } from "src/components/Connection/ConnectionEdit";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  tenantRoutePattern,
  testTenantApiUrl,
  testTenantPath,
} from "src/test/tenant_routes";
import { POSTGRES_CONNECTION } from "src/test/test_data";
import { UNAVAILABLE_CONNECTION_MESSAGE } from "src/components/Connection/constants";

const TYPE_DISABLED_MESSAGE =
  "PostgreSQL connections are turned off on this deployment";
const editPath = testTenantPath(
  APP_ROUTES.CONNECTION_EDIT.replace(":id", POSTGRES_CONNECTION.id),
);

describe("edit connection form", () => {
  test("shows the saved connection and sends the edited fields", async () => {
    let updatedConnection: unknown = null;
    worker.use(
      http.get(testTenantApiUrl(API_ENDPOINTS.CONNECTION.GET_BY_ID), () =>
        HttpResponse.json(POSTGRES_CONNECTION),
      ),
      http.put(
        testTenantApiUrl(API_ENDPOINTS.CONNECTION.UPDATE),
        async ({ request }) => {
          updatedConnection = await request.json();
          return HttpResponse.json(POSTGRES_CONNECTION);
        },
      ),
    );
    const { router } = await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.CONNECTION_EDIT),
          element: <ConnectionEdit />,
        },
        {
          path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
          element: <p>Connection list</p>,
        },
      ],
      initialPath: testTenantPath(
        APP_ROUTES.CONNECTION_EDIT.replace(":id", POSTGRES_CONNECTION.id),
      ),
    });

    await expect
      .element(page.getByPlaceholder("Connection name"))
      .toHaveValue(POSTGRES_CONNECTION.name);
    await expect
      .element(page.getByPlaceholder("localhost"))
      .toHaveValue(POSTGRES_CONNECTION.host);
    await expect.element(page.getByPlaceholder("5432")).toHaveValue("5432");
    await page.getByPlaceholder("Connection name").fill("Warehouse replica");
    await page.getByPlaceholder("5432").fill("6432");
    await page.getByRole("button", { name: "Save Changes" }).click();

    await expect.element(page.getByText("Connection list")).toBeVisible();
    expect(router.state.location.pathname).toBe(
      testTenantPath(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
    );
    await expect
      .poll(() => updatedConnection)
      .toEqual({
        name: "Warehouse replica",
        description: POSTGRES_CONNECTION.description,
        host: POSTGRES_CONNECTION.host,
        port: 6432,
        database: POSTGRES_CONNECTION.database,
        schema: POSTGRES_CONNECTION.schema,
        username: POSTGRES_CONNECTION.username,
        password: null,
      });
  });

  test("marks a connection whose type is turned off and shows the refusal", async () => {
    worker.use(
      http.get(testTenantApiUrl(API_ENDPOINTS.CONNECTION.GET_BY_ID), () =>
        HttpResponse.json({
          ...POSTGRES_CONNECTION,
          is_source_type_available: false,
        }),
      ),
      http.put(testTenantApiUrl(API_ENDPOINTS.CONNECTION.UPDATE), () =>
        HttpResponse.json(
          {
            error: TYPE_DISABLED_MESSAGE,
            error_code: "CONNECTION_TYPE_DISABLED",
          },
          { status: 403 },
        ),
      ),
    );
    const { router } = await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.CONNECTION_EDIT),
          element: <ConnectionEdit />,
        },
        {
          path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
          element: <p>Connection list</p>,
        },
      ],
      initialPath: editPath,
    });

    await expect
      .element(page.getByText(UNAVAILABLE_CONNECTION_MESSAGE))
      .toBeVisible();
    await page.getByRole("button", { name: "Save Changes" }).click();

    await expect.element(page.getByText(TYPE_DISABLED_MESSAGE)).toBeVisible();
    expect(router.state.location.pathname).toBe(editPath);
  });

  test("shows no unavailable banner for an available connection", async () => {
    worker.use(
      http.get(testTenantApiUrl(API_ENDPOINTS.CONNECTION.GET_BY_ID), () =>
        HttpResponse.json(POSTGRES_CONNECTION),
      ),
    );
    await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.CONNECTION_EDIT),
          element: <ConnectionEdit />,
        },
      ],
      initialPath: editPath,
    });

    await expect
      .element(page.getByPlaceholder("Connection name"))
      .toHaveValue(POSTGRES_CONNECTION.name);
    await expect
      .element(page.getByText(UNAVAILABLE_CONNECTION_MESSAGE))
      .not.toBeInTheDocument();
  });
});
