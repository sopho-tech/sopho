import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import {
  useMemberships,
  type TenantCreationCapabilityDto,
  type TenantMembershipDto,
} from "src/api/tenant/queries";
import { CollapsibleSidebar } from "src/components/design-system";
import { TenantSwitcher } from "src/components/Sidebar/TenantSwitcher";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import { TEST_TENANT_ID } from "src/test/test_data";
import { tenantRoutePattern, testTenantPath } from "src/test/tenant_routes";
import { tenantPath } from "src/utils/tenant";

const CREATED_TENANT_ID = "00000000-0000-4000-8000-000000000902";

const CURRENT_TENANT: TenantMembershipDto = {
  tenant_id: TEST_TENANT_ID,
  tenant_name: "Acme Analytics",
  role_name: "OWNER",
  role_display_name: "Owner",
  permissions: [],
};

function ignorePinnedChange() {}

function SwitcherInSidebar() {
  const memberships = useMemberships();
  if (memberships.data === undefined) return null;
  return (
    <CollapsibleSidebar
      isPinned={true}
      onPinnedChange={ignorePinnedChange}
      header={<TenantSwitcher />}
    >
      <p>Navigation</p>
    </CollapsibleSidebar>
  );
}

const switcherRoutes = [
  {
    path: tenantRoutePattern(APP_ROUTES.INDEX),
    element: <SwitcherInSidebar />,
  },
];

function switcherHandlers(capability: TenantCreationCapabilityDto) {
  return [
    http.get(API_ENDPOINTS.INVITATION.RECEIVED, () => HttpResponse.json([])),
    http.get(API_ENDPOINTS.TENANT.LIST, () =>
      HttpResponse.json([CURRENT_TENANT]),
    ),
    http.get(API_ENDPOINTS.TENANT.CREATION_CAPABILITY, () =>
      HttpResponse.json(capability),
    ),
  ];
}

async function openSwitcher() {
  await page.getByRole("button", { name: "Tenant: Acme Analytics" }).click();
  await expect
    .element(page.getByRole("menuitemradio", { name: "Acme Analytics" }))
    .toBeVisible();
}

describe("tenant switcher", () => {
  test("offers tenant creation and opens the new tenant", async () => {
    worker.use(
      ...switcherHandlers({
        can_create_tenant: true,
        limit_reached_message: null,
      }),
      http.post(API_ENDPOINTS.TENANT.CREATE, () =>
        HttpResponse.json(
          { id: CREATED_TENANT_ID, name: "Side project" },
          { status: 201 },
        ),
      ),
    );
    const { router } = await renderWithProviders({
      routes: switcherRoutes,
      initialPath: testTenantPath(APP_ROUTES.INDEX),
    });

    await openSwitcher();
    await page.getByRole("menuitem", { name: "Create a tenant" }).click();
    await page.getByLabelText("Name", { exact: true }).fill("Side project");
    await page
      .getByRole("button", { name: "Create tenant", exact: true })
      .click();

    await expect
      .poll(() => router.state.location.pathname)
      .toBe(tenantPath(CREATED_TENANT_ID, APP_ROUTES.INDEX));
  });

  test("hides tenant creation when the backend does not allow it", async () => {
    worker.use(
      ...switcherHandlers({
        can_create_tenant: false,
        limit_reached_message: "You have created the maximum of 2 tenants",
      }),
    );
    await renderWithProviders({
      routes: switcherRoutes,
      initialPath: testTenantPath(APP_ROUTES.INDEX),
    });

    await openSwitcher();

    await expect
      .element(page.getByRole("menuitem", { name: "Create a tenant" }))
      .not.toBeInTheDocument();
  });
});
