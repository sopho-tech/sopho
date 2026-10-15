import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { ConnectionNew } from "src/components/Connection/ConnectionNew";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import { SQLITE_CONNECTION } from "src/test/test_data";

describe("new connection form", () => {
  test("creates a SQLite connection and returns to the connection list", async () => {
    let createdConnection: unknown = null;
    worker.use(
      http.post(API_ENDPOINTS.CONNECTION.CREATE, async ({ request }) => {
        createdConnection = await request.json();
        return HttpResponse.json(SQLITE_CONNECTION, { status: 201 });
      }),
    );
    const { router } = await renderWithProviders({
      routes: [
        { path: APP_ROUTES.CONNECTION_NEW, element: <ConnectionNew /> },
        {
          path: APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS,
          element: <p>Connection list</p>,
        },
      ],
      initialPath: APP_ROUTES.CONNECTION_NEW,
    });

    await page.getByPlaceholder("Connection name").fill(SQLITE_CONNECTION.name);
    await page.getByRole("combobox").click();
    await page.getByRole("option", { name: "Sqlite" }).click();
    await page
      .getByPlaceholder("/Users/username/database.db")
      .fill(SQLITE_CONNECTION.database);
    await page.getByRole("button", { name: "Save" }).click();

    await expect.element(page.getByText("Connection list")).toBeVisible();
    expect(router.state.location.pathname).toBe(
      APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS,
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

  test("asks for the PostgreSQL connection details", async () => {
    await renderWithProviders({
      routes: [{ path: APP_ROUTES.CONNECTION_NEW, element: <ConnectionNew /> }],
      initialPath: APP_ROUTES.CONNECTION_NEW,
    });

    await page.getByRole("combobox").click();
    await page.getByRole("option", { name: "Postgresql" }).click();
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
