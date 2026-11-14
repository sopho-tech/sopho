import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type {
  TenantCreationCapabilityDto,
  TenantMembershipDto,
} from "src/api/tenant/queries";
import { Onboarding } from "src/components/Onboarding";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import {
  currentUserHandler,
  signedInSessionHandler,
} from "src/test/msw/handlers";
import { renderWithProviders } from "src/test/render_with_providers";
import { TEST_TENANT_ID, TEST_USER } from "src/test/test_data";
import { testTenantPath } from "src/test/tenant_routes";

const LIMIT_REACHED_MESSAGE = "You have created the maximum of 2 tenants";

const OWNED_TENANT: TenantMembershipDto = {
  tenant_id: TEST_TENANT_ID,
  tenant_name: "Acme Analytics",
  role_name: "OWNER",
  role_display_name: "Owner",
  permissions: [],
  suspended_message: null,
};

const onboardingRoutes = [
  {
    element: <ProtectedRoute />,
    children: [
      { path: APP_ROUTES.ONBOARDING, element: <Onboarding /> },
      { path: APP_ROUTES.TENANT, element: <p>Tenant home</p> },
    ],
  },
];

function onboardingHandlers(
  memberships: TenantMembershipDto[],
  capability: TenantCreationCapabilityDto,
) {
  return [
    signedInSessionHandler,
    currentUserHandler,
    http.get(API_ENDPOINTS.INVITATION.RECEIVED, () => HttpResponse.json([])),
    http.get(API_ENDPOINTS.TENANT.LIST, () => HttpResponse.json(memberships)),
    http.get(API_ENDPOINTS.TENANT.CREATION_CAPABILITY, () =>
      HttpResponse.json(capability),
    ),
  ];
}

describe("onboarding", () => {
  test("a person without a tenant names one and lands in it", async () => {
    let submittedTenant: unknown = null;
    worker.use(
      ...onboardingHandlers([], {
        can_create_tenant: true,
        limit_reached_message: null,
      }),
      http.post(API_ENDPOINTS.TENANT.CREATE, async ({ request }) => {
        submittedTenant = await request.json();
        return HttpResponse.json(
          { id: TEST_TENANT_ID, name: "Acme Analytics" },
          { status: 201 },
        );
      }),
    );
    const { router } = await renderWithProviders({
      routes: onboardingRoutes,
      initialPath: APP_ROUTES.ONBOARDING,
    });

    await expect
      .element(page.getByLabelText("Enter tenant name"))
      .toBeVisible();
    await expect
      .element(page.getByText(`Ask a tenant admin to invite`))
      .not.toBeInTheDocument();
    const createButton = page.getByRole("button", {
      name: "Create tenant",
      exact: true,
    });
    await expect
      .element(page.getByRole("button", { name: "Sign out" }))
      .toBeVisible();
    await expect.element(createButton).toBeDisabled();

    await page.getByLabelText("Enter tenant name").fill("  Acme Analytics  ");
    await createButton.click();

    await expect.element(page.getByText("Tenant home")).toBeVisible();
    expect(router.state.location.pathname).toBe(
      testTenantPath(APP_ROUTES.INDEX),
    );
    expect(submittedTenant).toEqual({ name: "Acme Analytics" });
  });

  test("the name form stays in place until the new tenant opens", async () => {
    let isTenantCreated = false;
    let membershipRequestCount = 0;
    let releaseCapabilityRefetch: () => void = () => {};
    const capabilityRefetchReleased = new Promise<void>((resolve) => {
      releaseCapabilityRefetch = resolve;
    });
    worker.use(
      signedInSessionHandler,
      currentUserHandler,
      http.get(API_ENDPOINTS.INVITATION.RECEIVED, () => HttpResponse.json([])),
      http.get(API_ENDPOINTS.TENANT.LIST, () => {
        membershipRequestCount += 1;
        return HttpResponse.json(isTenantCreated ? [OWNED_TENANT] : []);
      }),
      http.get(API_ENDPOINTS.TENANT.CREATION_CAPABILITY, async () => {
        if (isTenantCreated) {
          await capabilityRefetchReleased;
        }
        return HttpResponse.json({
          can_create_tenant: true,
          limit_reached_message: null,
        });
      }),
      http.post(API_ENDPOINTS.TENANT.CREATE, () => {
        isTenantCreated = true;
        return HttpResponse.json(
          { id: TEST_TENANT_ID, name: "Acme Analytics" },
          { status: 201 },
        );
      }),
    );
    await renderWithProviders({
      routes: onboardingRoutes,
      initialPath: APP_ROUTES.ONBOARDING,
    });

    await page.getByLabelText("Enter tenant name").fill("Acme Analytics");
    await page
      .getByRole("button", { name: "Create tenant", exact: true })
      .click();
    await expect.poll(() => membershipRequestCount).toBe(2);
    await new Promise((resolve) => setTimeout(resolve, 200));

    await expect
      .element(page.getByLabelText("Enter tenant name"))
      .toBeVisible();
    await expect
      .element(page.getByText("Your tenants"))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: "Create a tenant" }))
      .not.toBeInTheDocument();

    releaseCapabilityRefetch();
    await expect.element(page.getByText("Tenant home")).toBeVisible();
  });

  test("a person at the limit sees why they cannot create another", async () => {
    worker.use(
      ...onboardingHandlers([OWNED_TENANT], {
        can_create_tenant: false,
        limit_reached_message: LIMIT_REACHED_MESSAGE,
      }),
    );
    await renderWithProviders({
      routes: onboardingRoutes,
      initialPath: APP_ROUTES.ONBOARDING,
    });

    await expect.element(page.getByText(LIMIT_REACHED_MESSAGE)).toBeVisible();
    await expect.element(page.getByText("Acme Analytics")).toBeVisible();
    await expect
      .element(page.getByRole("button", { name: "Create a tenant" }))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByLabelText("Enter tenant name"))
      .not.toBeInTheDocument();
  });

  test("a person who may not create a tenant is told to ask for an invitation", async () => {
    worker.use(
      ...onboardingHandlers([], {
        can_create_tenant: false,
        limit_reached_message: null,
      }),
    );
    await renderWithProviders({
      routes: onboardingRoutes,
      initialPath: APP_ROUTES.ONBOARDING,
    });

    await expect
      .element(
        page.getByText(
          `You are not part of a tenant yet. Ask a tenant admin to invite ${TEST_USER.email}.`,
        ),
      )
      .toBeVisible();
    await expect
      .element(page.getByLabelText("Enter tenant name"))
      .not.toBeInTheDocument();
  });

  test("a person with a tenant can open the dialog to create another", async () => {
    worker.use(
      ...onboardingHandlers([OWNED_TENANT], {
        can_create_tenant: true,
        limit_reached_message: null,
      }),
    );
    await renderWithProviders({
      routes: onboardingRoutes,
      initialPath: APP_ROUTES.ONBOARDING,
    });

    await page.getByRole("button", { name: "Create a tenant" }).click();

    await expect
      .element(page.getByRole("dialog").getByLabelText("Name", { exact: true }))
      .toBeVisible();
  });

  test("marks a suspended tenant in the list", async () => {
    worker.use(
      ...onboardingHandlers(
        [
          OWNED_TENANT,
          {
            ...OWNED_TENANT,
            tenant_id: "00000000-0000-4000-8000-000000000903",
            tenant_name: "Frozen Works",
            suspended_message: "This tenant is suspended. Contact support",
          },
        ],
        { can_create_tenant: false, limit_reached_message: null },
      ),
    );
    await renderWithProviders({
      routes: onboardingRoutes,
      initialPath: APP_ROUTES.ONBOARDING,
    });

    await expect.element(page.getByText("Frozen Works")).toBeVisible();
    await expect.element(page.getByText("Acme Analytics")).toBeVisible();
    expect(
      page.getByText("Suspended", { exact: true }).elements(),
    ).toHaveLength(1);
  });
});
