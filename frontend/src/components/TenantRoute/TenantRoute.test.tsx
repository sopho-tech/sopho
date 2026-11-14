import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type { TenantMembershipDto } from "src/api/tenant/queries";
import { TenantRoute } from "src/components/TenantRoute/TenantRoute";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import { TEST_TENANT_ID } from "src/test/test_data";
import { testTenantPath } from "src/test/tenant_routes";

const SUSPENDED_MESSAGE = "This tenant is suspended. Contact support";

const SUSPENDED_TENANT: TenantMembershipDto = {
  tenant_id: TEST_TENANT_ID,
  tenant_name: "Frozen Works",
  role_name: "OWNER",
  role_display_name: "Owner",
  permissions: [],
  suspended_message: SUSPENDED_MESSAGE,
};

const tenantRoutes = [
  {
    path: APP_ROUTES.TENANT,
    element: <TenantRoute />,
    children: [{ index: true, element: <p>Tenant home</p> }],
  },
  { path: APP_ROUTES.ONBOARDING, element: <p>Tenant chooser</p> },
];

describe("tenant route", () => {
  test("shows the backend's suspension message instead of the tenant", async () => {
    worker.use(
      http.get(API_ENDPOINTS.TENANT.LIST, () =>
        HttpResponse.json([SUSPENDED_TENANT]),
      ),
    );
    const { router } = await renderWithProviders({
      routes: tenantRoutes,
      initialPath: testTenantPath(APP_ROUTES.INDEX),
    });

    await expect.element(page.getByText(SUSPENDED_MESSAGE)).toBeVisible();
    await expect
      .element(page.getByRole("heading", { name: "Frozen Works" }))
      .toBeVisible();
    await expect.element(page.getByText("Tenant home")).not.toBeInTheDocument();

    await page.getByRole("button", { name: "Back to your tenants" }).click();

    await expect.element(page.getByText("Tenant chooser")).toBeVisible();
    expect(router.state.location.pathname).toBe(APP_ROUTES.ONBOARDING);
  });
});
