import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import {
  useMemberships,
  type TenantMembershipDto,
} from "src/api/tenant/queries";
import { ConnectionsTable } from "src/components/Connection/ConnectionsTable";
import { UNAVAILABLE_CONNECTION_STATUS_LABEL } from "src/components/Connection/constants";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  tenantRoutePattern,
  testTenantApiUrl,
  testTenantPath,
} from "src/test/tenant_routes";
import {
  POSTGRES_CONNECTION,
  SQLITE_CONNECTION,
  TEST_TENANT_ID,
} from "src/test/test_data";

const CURRENT_TENANT: TenantMembershipDto = {
  tenant_id: TEST_TENANT_ID,
  tenant_name: "Acme Analytics",
  role_name: "OWNER",
  role_display_name: "Owner",
  permissions: [],
  suspended_message: null,
};

function ConnectionsTableOnceMembershipLoads() {
  const memberships = useMemberships();
  if (memberships.data === undefined) return null;
  return <ConnectionsTable />;
}

async function renderConnectionsTable() {
  worker.use(
    http.get(API_ENDPOINTS.TENANT.LIST, () =>
      HttpResponse.json([CURRENT_TENANT]),
    ),
  );
  await renderWithProviders({
    routes: [
      {
        path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
        element: <ConnectionsTableOnceMembershipLoads />,
      },
    ],
    initialPath: testTenantPath(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
  });
}

describe("connections table", () => {
  test("marks a connection whose type is turned off as unavailable", async () => {
    worker.use(
      http.get(testTenantApiUrl(API_ENDPOINTS.CONNECTION.GET_ALL), () =>
        HttpResponse.json([
          POSTGRES_CONNECTION,
          { ...SQLITE_CONNECTION, is_source_type_available: false },
        ]),
      ),
    );

    await renderConnectionsTable();

    await expect.element(page.getByText("PostgreSQL")).toBeVisible();
    await expect.element(page.getByText("SQLite")).toBeVisible();
    await expect
      .element(page.getByText(UNAVAILABLE_CONNECTION_STATUS_LABEL))
      .toBeVisible();
    await expect.element(page.getByText("ACTIVE")).toBeVisible();
  });

  test("shows the stored status for available connections", async () => {
    worker.use(
      http.get(testTenantApiUrl(API_ENDPOINTS.CONNECTION.GET_ALL), () =>
        HttpResponse.json([POSTGRES_CONNECTION]),
      ),
    );

    await renderConnectionsTable();

    await expect.element(page.getByText("ACTIVE")).toBeVisible();
    await expect
      .element(page.getByText(UNAVAILABLE_CONNECTION_STATUS_LABEL))
      .not.toBeInTheDocument();
  });
});
