import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { ConnectionEdit } from "src/components/Connection/ConnectionEdit";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import { POSTGRES_CONNECTION } from "src/test/test_data";

describe("edit connection form", () => {
  test("shows the saved connection and sends the edited fields", async () => {
    let updatedConnection: unknown = null;
    worker.use(
      http.get(API_ENDPOINTS.CONNECTION.GET_BY_ID, () =>
        HttpResponse.json(POSTGRES_CONNECTION),
      ),
      http.put(API_ENDPOINTS.CONNECTION.UPDATE, async ({ request }) => {
        updatedConnection = await request.json();
        return HttpResponse.json(POSTGRES_CONNECTION);
      }),
    );
    const { router } = await renderWithProviders({
      routes: [
        { path: APP_ROUTES.CONNECTION_EDIT, element: <ConnectionEdit /> },
        {
          path: APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS,
          element: <p>Connection list</p>,
        },
      ],
      initialPath: APP_ROUTES.CONNECTION_EDIT.replace(
        ":id",
        POSTGRES_CONNECTION.id,
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
      APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS,
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
});
