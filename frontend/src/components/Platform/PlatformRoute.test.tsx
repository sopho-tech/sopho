import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import {
  useMemberships,
  type TenantMembershipDto,
} from "src/api/tenant/queries";
import {
  PlatformIndexRedirect,
  PlatformRoute,
} from "src/components/Platform/PlatformRoute";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { Settings } from "src/components/Settings";
import { TenantRoute } from "src/components/TenantRoute";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import {
  PLATFORM_PERMISSIONS,
  type PlatformPermission,
} from "src/constants/platform_permissions";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import { tenantRoutePattern, testTenantPath } from "src/test/tenant_routes";
import { useStore } from "src/store";
import { TEST_TENANT_ID, TEST_USER } from "src/test/test_data";

const ALL_PLATFORM_PERMISSIONS: PlatformPermission[] = [
  PLATFORM_PERMISSIONS.USER_MANAGE,
  PLATFORM_PERMISSIONS.TENANT_CREATE,
  PLATFORM_PERMISSIONS.EMAIL_MANAGE,
  PLATFORM_PERMISSIONS.PLATFORM_AI_USAGE_READ,
];

const CURRENT_TENANT: TenantMembershipDto = {
  tenant_id: TEST_TENANT_ID,
  tenant_name: "Acme Analytics",
  role_name: "OWNER",
  role_display_name: "Owner",
  permissions: [],
  suspended_message: null,
};

function signedInWith(platformPermissions: PlatformPermission[]) {
  return [
    http.get(API_ENDPOINTS.AUTH.GET_PROFILE_DETAILS, () =>
      HttpResponse.json({
        ...TEST_USER,
        platform_permissions: platformPermissions,
      }),
    ),
    http.get(API_ENDPOINTS.TENANT.LIST, () =>
      HttpResponse.json([CURRENT_TENANT]),
    ),
    http.get(API_ENDPOINTS.INVITATION.RECEIVED, () => HttpResponse.json([])),
    http.get(API_ENDPOINTS.TENANT.CREATION_CAPABILITY, () =>
      HttpResponse.json({
        can_create_tenant: false,
        limit_reached_message: null,
      }),
    ),
  ];
}

function SettingsOnceMembershipLoads() {
  const memberships = useMemberships();
  if (memberships.data === undefined) return null;
  return <Settings />;
}

const routes = [
  {
    element: <ProtectedRoute />,
    children: [
      { path: APP_ROUTES.INDEX, element: <p>Tenant chooser</p> },
      {
        path: APP_ROUTES.PLATFORM,
        element: <PlatformRoute />,
        children: [
          { index: true, element: <PlatformIndexRedirect /> },
          { path: "users", element: <p>Users page</p> },
        ],
      },
      {
        path: tenantRoutePattern(APP_ROUTES.INDEX),
        element: <TenantRoute />,
      },
      {
        path: tenantRoutePattern(APP_ROUTES.SETTINGS),
        element: <SettingsOnceMembershipLoads />,
      },
    ],
  },
];

describe("platform area", () => {
  test("opens on its first page with its own menu and no tenant in the address", async () => {
    worker.use(...signedInWith(ALL_PLATFORM_PERMISSIONS));

    const { router } = await renderWithProviders({
      routes,
      initialPath: APP_ROUTES.PLATFORM,
    });

    await expect.element(page.getByText("Users page")).toBeVisible();
    expect(router.state.location.pathname).toBe(
      APP_ROUTES.PLATFORM_ROUTES.USERS,
    );
    await expect
      .element(page.getByRole("link", { name: "Users", exact: true }))
      .toBeVisible();
    await expect
      .element(page.getByRole("link", { name: "Tenants", exact: true }))
      .toHaveAttribute("href", APP_ROUTES.PLATFORM_ROUTES.TENANTS);
    await expect
      .element(page.getByRole("link", { name: "AI Usage", exact: true }))
      .toHaveAttribute("href", APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_SUMMARY);
    await expect
      .element(page.getByRole("link", { name: "Sent", exact: true }))
      .toHaveAttribute("href", APP_ROUTES.PLATFORM_ROUTES.SENT_EMAILS);
    await expect.element(page.getByText(/Global/)).not.toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: /Tenant:/ }))
      .not.toBeInTheDocument();
  });

  test("has no application sidebar, only its own menu", async () => {
    worker.use(...signedInWith(ALL_PLATFORM_PERMISSIONS));
    await renderWithProviders({
      routes,
      initialPath: APP_ROUTES.PLATFORM_ROUTES.USERS,
    });

    await expect.element(page.getByText("Users page")).toBeVisible();
    await expect
      .element(page.getByRole("link", { name: "Profile" }))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByRole("link", { name: "Platform", exact: true }))
      .not.toBeInTheDocument();
  });

  test("exits to the tenant the person was last in", async () => {
    worker.use(...signedInWith(ALL_PLATFORM_PERMISSIONS));
    const { router } = await renderWithProviders({
      routes,
      initialPath: testTenantPath(APP_ROUTES.INDEX),
    });
    await page.getByRole("link", { name: "Platform", exact: true }).click();

    await page.getByRole("button", { name: "Exit platform" }).click();

    await expect
      .poll(() => router.state.location.pathname)
      .toBe(testTenantPath(APP_ROUTES.INDEX));
  });

  test("exits to the tenant chooser when no tenant was opened before", async () => {
    worker.use(...signedInWith(ALL_PLATFORM_PERMISSIONS));
    window.localStorage.clear();
    useStore.setState((state) => ({
      tenant: { ...state.tenant, lastTenantId: null },
    }));
    await renderWithProviders({
      routes,
      initialPath: APP_ROUTES.PLATFORM_ROUTES.USERS,
    });

    await page.getByRole("button", { name: "Exit platform" }).click();

    await expect.element(page.getByText("Tenant chooser")).toBeVisible();
  });

  test("shows only the pages the person's platform permissions allow", async () => {
    worker.use(...signedInWith([PLATFORM_PERMISSIONS.USER_MANAGE]));

    await renderWithProviders({
      routes,
      initialPath: APP_ROUTES.PLATFORM_ROUTES.USERS,
    });

    await expect
      .element(page.getByRole("link", { name: "Users", exact: true }))
      .toBeVisible();
    await expect
      .element(page.getByRole("link", { name: "Tenants", exact: true }))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByRole("link", { name: "AI Usage", exact: true }))
      .not.toBeInTheDocument();
  });

  test("turns away someone with no platform permission", async () => {
    worker.use(...signedInWith([]));

    const { router } = await renderWithProviders({
      routes,
      initialPath: APP_ROUTES.PLATFORM_ROUTES.USERS,
    });

    await expect.element(page.getByText("Tenant chooser")).toBeVisible();
    expect(router.state.location.pathname).toBe(APP_ROUTES.INDEX);
  });

  test("is offered in the sidebar only to someone with a platform permission", async () => {
    worker.use(...signedInWith([PLATFORM_PERMISSIONS.EMAIL_MANAGE]));
    const { router } = await renderWithProviders({
      routes,
      initialPath: testTenantPath(APP_ROUTES.INDEX),
    });

    await page.getByRole("link", { name: "Platform", exact: true }).click();

    await expect
      .poll(() => router.state.location.pathname)
      .toBe(APP_ROUTES.PLATFORM_ROUTES.SENT_EMAILS);
  });

  test("is not offered in the sidebar without a platform permission", async () => {
    worker.use(...signedInWith([]));

    await renderWithProviders({
      routes,
      initialPath: testTenantPath(APP_ROUTES.INDEX),
    });

    await expect
      .element(page.getByRole("link", { name: "Settings" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("link", { name: "Platform", exact: true }))
      .not.toBeInTheDocument();
  });

  test("leaves only tenant pages in Settings", async () => {
    worker.use(...signedInWith(ALL_PLATFORM_PERMISSIONS));

    await renderWithProviders({
      routes,
      initialPath: testTenantPath(APP_ROUTES.SETTINGS),
    });

    await expect
      .element(page.getByRole("link", { name: "Connections" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("link", { name: /Users|Tenants|Emails/ }))
      .not.toBeInTheDocument();
  });
});
