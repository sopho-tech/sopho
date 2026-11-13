import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { ConnectionDetail } from "src/components/Connection/ConnectionDetail";
import { UNAVAILABLE_CONNECTION_MESSAGE } from "src/components/Connection/constants";
import {
  ConnectionDetailsPageStateEnum,
  type ConnectionDto,
} from "src/components/Connection/dto";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { useStore } from "src/store";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  tenantRoutePattern,
  testTenantApiUrl,
  testTenantPath,
} from "src/test/tenant_routes";
import { POSTGRES_CONNECTION } from "src/test/test_data";

async function openConnectionDetail(connection: ConnectionDto) {
  worker.use(
    http.get(testTenantApiUrl(API_ENDPOINTS.CONNECTION.GET_BY_ID), () =>
      HttpResponse.json(connection),
    ),
  );
  await renderWithProviders({
    routes: [
      {
        path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
        element: <ConnectionDetail />,
      },
    ],
    initialPath: testTenantPath(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
  });
  const connectionStore = useStore.getState().connection;
  connectionStore.setConnectionId(connection.id);
  connectionStore.setConnectionDetailsPageState(
    ConnectionDetailsPageStateEnum.DETAIL,
  );
  await expect.element(page.getByText("Connection Details")).toBeVisible();
}

describe("connection details", () => {
  test("marks a connection whose type is turned off", async () => {
    await openConnectionDetail({
      ...POSTGRES_CONNECTION,
      is_source_type_available: false,
    });

    await expect
      .element(page.getByText(UNAVAILABLE_CONNECTION_MESSAGE))
      .toBeVisible();
  });

  test("shows the backend's name for the connection type without a warning", async () => {
    await openConnectionDetail(POSTGRES_CONNECTION);

    await expect
      .element(page.getByRole("textbox", { name: "Source Type" }))
      .toHaveValue(POSTGRES_CONNECTION.source_type_display_name);
    await expect
      .element(page.getByText(UNAVAILABLE_CONNECTION_MESSAGE))
      .not.toBeInTheDocument();
  });
});
