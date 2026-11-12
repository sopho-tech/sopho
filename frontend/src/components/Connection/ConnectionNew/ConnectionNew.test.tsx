import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { ConnectionNew } from "src/components/Connection/ConnectionNew";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  tenantRoutePattern,
  testTenantApiUrl,
  testTenantPath,
} from "src/test/tenant_routes";
import { SQLITE_CONNECTION } from "src/test/test_data";

const HOST_NOT_ALLOWED_MESSAGE =
  "This host is not reachable from this deployment";

describe("new connection form", () => {
  test("creates a SQLite connection and returns to the connection list", async () => {
    let createdConnection: unknown = null;
    worker.use(
      http.post(
        testTenantApiUrl(API_ENDPOINTS.CONNECTION.CREATE),
        async ({ request }) => {
          createdConnection = await request.json();
          return HttpResponse.json(SQLITE_CONNECTION, { status: 201 });
        },
      ),
    );
    const { router } = await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.CONNECTION_NEW),
          element: <ConnectionNew />,
        },
        {
          path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
          element: <p>Connection list</p>,
        },
      ],
      initialPath: testTenantPath(APP_ROUTES.CONNECTION_NEW),
    });

    await page.getByPlaceholder("Connection name").fill(SQLITE_CONNECTION.name);
    await page.getByRole("combobox").click();
    await page.getByRole("option", { name: "SQLite" }).click();
    await page
      .getByPlaceholder("/Users/username/database.db")
      .fill(SQLITE_CONNECTION.database);
    await page.getByRole("button", { name: "Save" }).click();

    await expect.element(page.getByText("Connection list")).toBeVisible();
    expect(router.state.location.pathname).toBe(
      testTenantPath(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
    );
    await expect
      .poll(() => createdConnection)
      .toEqual({
        name: SQLITE_CONNECTION.name,
        source_type: "SQLITE",
        port: null,
        database: SQLITE_CONNECTION.database,
        schema: null,
        description: null,
        status: "ACTIVE",
      });
  });

  test("offers only the connection types the backend returns", async () => {
    worker.use(
      http.get(
        testTenantApiUrl(API_ENDPOINTS.CONNECTION.OFFERED_SOURCE_TYPES),
        () =>
          HttpResponse.json([
            { source_type: "POSTGRESQL", display_name: "PostgreSQL" },
            { source_type: "SUPABASE", display_name: "Supabase" },
          ]),
      ),
    );
    await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.CONNECTION_NEW),
          element: <ConnectionNew />,
        },
      ],
      initialPath: testTenantPath(APP_ROUTES.CONNECTION_NEW),
    });

    await page.getByRole("combobox").click();

    await expect
      .element(page.getByRole("option", { name: "PostgreSQL" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("option", { name: "Supabase" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("option", { name: "SQLite" }))
      .not.toBeInTheDocument();
  });

  test("shows the backend's message and stays on the form when the connection is refused", async () => {
    worker.use(
      http.post(testTenantApiUrl(API_ENDPOINTS.CONNECTION.CREATE), () =>
        HttpResponse.json(
          {
            error: HOST_NOT_ALLOWED_MESSAGE,
            error_code: "CONNECTION_HOST_NOT_ALLOWED",
          },
          { status: 403 },
        ),
      ),
    );
    const { router } = await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.CONNECTION_NEW),
          element: <ConnectionNew />,
        },
        {
          path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
          element: <p>Connection list</p>,
        },
      ],
      initialPath: testTenantPath(APP_ROUTES.CONNECTION_NEW),
    });

    await page.getByPlaceholder("Connection name").fill("Warehouse");
    await page.getByRole("combobox").click();
    await page.getByRole("option", { name: "PostgreSQL" }).click();
    await page.getByPlaceholder("admin").fill("analyst");
    await page.getByPlaceholder("toughpassword@123").fill("secret");
    await page.getByPlaceholder("localhost").fill("10.0.0.5");
    await page.getByPlaceholder("5432").fill("5432");
    await page.getByPlaceholder("my_database").fill("sales");
    await page.getByRole("button", { name: "Save" }).click();

    await expect
      .element(page.getByText(HOST_NOT_ALLOWED_MESSAGE))
      .toBeVisible();
    expect(router.state.location.pathname).toBe(
      testTenantPath(APP_ROUTES.CONNECTION_NEW),
    );
  });

  test("sends one request when Save is clicked again while the first is pending", async () => {
    let createRequestCount = 0;
    let releaseResponse: () => void = () => {};
    const responseReleased = new Promise<void>((resolve) => {
      releaseResponse = resolve;
    });
    worker.use(
      http.post(testTenantApiUrl(API_ENDPOINTS.CONNECTION.CREATE), async () => {
        createRequestCount += 1;
        await responseReleased;
        return HttpResponse.json(SQLITE_CONNECTION, { status: 201 });
      }),
    );
    await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.CONNECTION_NEW),
          element: <ConnectionNew />,
        },
        {
          path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
          element: <p>Connection list</p>,
        },
      ],
      initialPath: testTenantPath(APP_ROUTES.CONNECTION_NEW),
    });

    await page.getByPlaceholder("Connection name").fill(SQLITE_CONNECTION.name);
    await page.getByRole("combobox").click();
    await page.getByRole("option", { name: "SQLite" }).click();
    await page
      .getByPlaceholder("/Users/username/database.db")
      .fill(SQLITE_CONNECTION.database);
    await page.getByRole("button", { name: "Save" }).click();

    await expect
      .element(page.getByRole("button", { name: "Save" }))
      .toBeDisabled();
    expect(createRequestCount).toBe(1);
    releaseResponse();
    await expect.element(page.getByText("Connection list")).toBeVisible();
    expect(createRequestCount).toBe(1);
  });

  test("asks for the PostgreSQL connection details", async () => {
    await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.CONNECTION_NEW),
          element: <ConnectionNew />,
        },
      ],
      initialPath: testTenantPath(APP_ROUTES.CONNECTION_NEW),
    });

    await page.getByRole("combobox").click();
    await page.getByRole("option", { name: "PostgreSQL" }).click();
    await page.getByRole("button", { name: "Save" }).click();

    await expect.element(page.getByText("Enter the username")).toBeVisible();
    await expect.element(page.getByText("Enter the password")).toBeVisible();
    await expect
      .element(page.getByText("Enter the database host name"))
      .toBeVisible();
    await expect
      .element(page.getByText("Enter the database port number"))
      .toBeVisible();
    await expect
      .element(page.getByText("Enter the database name"))
      .toBeVisible();
  });
});
