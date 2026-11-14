import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type { PlatformTenantDto } from "src/api/platform/queries";
import { Tenants } from "src/components/Platform/Tenants";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";
import { worker } from "src/test/msw/browser_worker";
import { signedInSessionHandler } from "src/test/msw/handlers";
import { renderWithProviders } from "src/test/render_with_providers";
import { TEST_USER } from "src/test/test_data";

const TENANTS_PATH = "/platform/tenants";
const ACTIVE_TENANT_ID = "00000000-0000-4000-8000-000000000911";
const SUSPENDED_TENANT_ID = "00000000-0000-4000-8000-000000000912";

const ACTIVE_TENANT: PlatformTenantDto = {
  tenant_id: ACTIVE_TENANT_ID,
  name: "Acme Analytics",
  status: "ACTIVE",
  capabilities: { can_suspend: true, can_restore: false },
  active_owner_count: 1,
  created_at: "2026-10-01T09:00:00Z",
};

const SUSPENDED_TENANT: PlatformTenantDto = {
  tenant_id: SUSPENDED_TENANT_ID,
  name: "Frozen Works",
  status: "SUSPENDED",
  capabilities: { can_suspend: false, can_restore: true },
  active_owner_count: 1,
  created_at: "2026-10-02T09:00:00Z",
};

const tenantsRoutes = [
  {
    element: <ProtectedRoute />,
    children: [{ path: TENANTS_PATH, element: <Tenants /> }],
  },
];

function platformHandlers(tenants: PlatformTenantDto[]) {
  return [
    signedInSessionHandler,
    http.get(API_ENDPOINTS.AUTH.GET_PROFILE_DETAILS, () =>
      HttpResponse.json({
        ...TEST_USER,
        platform_permissions: [
          PLATFORM_PERMISSIONS.TENANT_APPOINT_OWNER,
          PLATFORM_PERMISSIONS.TENANT_SUSPEND,
        ],
      }),
    ),
    http.get(API_ENDPOINTS.PLATFORM.TENANTS, () => HttpResponse.json(tenants)),
  ];
}

describe("platform tenants", () => {
  test("offers suspend or restore from the backend's capabilities", async () => {
    worker.use(...platformHandlers([ACTIVE_TENANT, SUSPENDED_TENANT]));
    await renderWithProviders({
      routes: tenantsRoutes,
      initialPath: TENANTS_PATH,
    });

    const activeRow = page.getByRole("row", { name: "Acme Analytics" });
    const suspendedRow = page.getByRole("row", { name: "Frozen Works" });
    await expect.element(activeRow.getByText("Active")).toBeVisible();
    await expect
      .element(activeRow.getByRole("button", { name: "Suspend" }))
      .toBeVisible();
    await expect
      .element(activeRow.getByRole("button", { name: "Restore" }))
      .not.toBeInTheDocument();
    await expect.element(suspendedRow.getByText("Suspended")).toBeVisible();
    await expect
      .element(suspendedRow.getByRole("button", { name: "Restore" }))
      .toBeVisible();
    await expect
      .element(suspendedRow.getByRole("button", { name: "Suspend" }))
      .not.toBeInTheDocument();
  });

  test("suspends a tenant only after confirmation", async () => {
    const suspendedTenantIds: string[] = [];
    worker.use(
      ...platformHandlers([ACTIVE_TENANT]),
      http.post(API_ENDPOINTS.PLATFORM.SUSPEND_TENANT, ({ params }) => {
        suspendedTenantIds.push(String(params.tenant_id));
        return new HttpResponse(null, { status: 204 });
      }),
    );
    await renderWithProviders({
      routes: tenantsRoutes,
      initialPath: TENANTS_PATH,
    });

    await page
      .getByRole("row", { name: "Acme Analytics" })
      .getByRole("button", { name: "Suspend" })
      .click();
    await expect
      .element(page.getByText("Suspend Acme Analytics?"))
      .toBeVisible();
    expect(suspendedTenantIds).toEqual([]);

    await page.getByRole("button", { name: "Cancel" }).click();
    await expect
      .element(page.getByText("Suspend Acme Analytics?"))
      .not.toBeInTheDocument();
    expect(suspendedTenantIds).toEqual([]);

    await page
      .getByRole("row", { name: "Acme Analytics" })
      .getByRole("button", { name: "Suspend" })
      .click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Suspend" })
      .click();

    await expect.poll(() => suspendedTenantIds).toEqual([ACTIVE_TENANT_ID]);
    await expect
      .element(page.getByText("Suspend Acme Analytics?"))
      .not.toBeInTheDocument();
  });

  test("restores a tenant and shows the backend's error when it fails", async () => {
    const restoredTenantIds: string[] = [];
    worker.use(
      ...platformHandlers([SUSPENDED_TENANT]),
      http.post(API_ENDPOINTS.PLATFORM.RESTORE_TENANT, ({ params }) => {
        restoredTenantIds.push(String(params.tenant_id));
        return HttpResponse.json(
          { message: "Tenant not found", error_code: "TENANT_NOT_FOUND" },
          { status: 404 },
        );
      }),
    );
    await renderWithProviders({
      routes: tenantsRoutes,
      initialPath: TENANTS_PATH,
    });

    await page
      .getByRole("row", { name: "Frozen Works" })
      .getByRole("button", { name: "Restore" })
      .click();
    await expect.element(page.getByText("Restore Frozen Works?")).toBeVisible();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Restore" })
      .click();

    await expect.poll(() => restoredTenantIds).toEqual([SUSPENDED_TENANT_ID]);
    await expect.element(page.getByText("Tenant not found")).toBeVisible();
    await expect.element(page.getByText("Restore Frozen Works?")).toBeVisible();
  });
});
