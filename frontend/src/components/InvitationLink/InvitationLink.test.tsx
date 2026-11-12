import { http, HttpResponse } from "msw";
import { afterEach, describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type { InvitationLinkDto } from "src/api/invitation_link/queries";
import { InvitationLink } from "src/components/InvitationLink";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { TenantIndexRedirect } from "src/components/TenantRoute";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import { TEST_TENANT_ID } from "src/test/test_data";
import {
  invitationPath,
  rememberPendingInvitation,
} from "src/components/InvitationLink/pending_invitation";

const DEAD_INVITATION_TOKEN = "dead-invitation-token";
const DEAD_INVITATION_MESSAGE =
  "This invitation is no longer valid. Ask the person who invited you for a new one";

const NEW_PERSON_INVITATION_TOKEN = "new-person-invitation-token";

function newPersonInvitation(
  isPasswordSignUpAvailable: boolean,
): InvitationLinkDto {
  return {
    tenant_name: "Acme",
    role_name: "Member",
    invited_by_name: "The Owner",
    email: "newcomer@example.test",
    expires_at: "2026-12-01T09:00:00Z",
    next_step: "SIGN_UP",
    password_sign_up_available: isPasswordSignUpAvailable,
  };
}

const invitationRoutes = [
  {
    element: <ProtectedRoute />,
    children: [
      { path: APP_ROUTES.INDEX, element: <TenantIndexRedirect /> },
      { path: APP_ROUTES.TENANT, element: <p>Tenant home</p> },
    ],
  },
  { path: APP_ROUTES.INVITATION, element: <InvitationLink /> },
];

describe("invitation link", () => {
  afterEach(() => {
    window.sessionStorage.clear();
  });

  test("going home after a dead invitation link stays home", async () => {
    worker.use(
      http.post(API_ENDPOINTS.INVITATION_LINK.LOOKUP, () =>
        HttpResponse.json(
          {
            message: DEAD_INVITATION_MESSAGE,
            error_code: "INVITATION_LINK_INVALID",
          },
          { status: 404 },
        ),
      ),
      http.get(API_ENDPOINTS.INVITATION.RECEIVED, () => HttpResponse.json([])),
      http.get(API_ENDPOINTS.TENANT.LIST, () =>
        HttpResponse.json([
          {
            tenant_id: TEST_TENANT_ID,
            tenant_name: "Acme",
            role_name: "OWNER",
            permissions: [],
          },
        ]),
      ),
    );
    rememberPendingInvitation(DEAD_INVITATION_TOKEN);
    const { router } = await renderWithProviders({
      routes: invitationRoutes,
      initialPath: invitationPath(DEAD_INVITATION_TOKEN),
    });
    await expect.element(page.getByText(DEAD_INVITATION_MESSAGE)).toBeVisible();

    await router.navigate(APP_ROUTES.INDEX);

    await expect.element(page.getByText("Tenant home")).toBeVisible();
  });
  test("a new person joins only through a provider when password sign-up is unavailable", async () => {
    worker.use(
      http.post(API_ENDPOINTS.INVITATION_LINK.LOOKUP, () =>
        HttpResponse.json(newPersonInvitation(false)),
      ),
      http.get(API_ENDPOINTS.AUTH.SIGN_IN_PAGE_OPTIONS, () =>
        HttpResponse.json({
          providers: [
            { name: "google", display_name: "Google", icon: "GOOGLE" },
          ],
          password_sign_in_available: false,
          password_reset_available: false,
        }),
      ),
    );
    await renderWithProviders({
      routes: invitationRoutes,
      initialPath: invitationPath(NEW_PERSON_INVITATION_TOKEN),
    });

    await expect
      .element(page.getByRole("heading", { name: "Join Acme" }))
      .toBeVisible();
    await expect
      .element(
        page.getByRole("button", { name: "Continue with Google", exact: true }),
      )
      .toBeVisible();
    await expect
      .element(page.getByRole("button", { name: "Create account" }))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByPlaceholder("Choose a password"))
      .not.toBeInTheDocument();
  });

  test("a new person can create a password account when password sign-up is available", async () => {
    worker.use(
      http.post(API_ENDPOINTS.INVITATION_LINK.LOOKUP, () =>
        HttpResponse.json(newPersonInvitation(true)),
      ),
    );
    await renderWithProviders({
      routes: invitationRoutes,
      initialPath: invitationPath(NEW_PERSON_INVITATION_TOKEN),
    });

    await expect
      .element(page.getByRole("button", { name: "Create account" }))
      .toBeVisible();
    await expect
      .element(page.getByPlaceholder("Choose a password"))
      .toBeVisible();
  });
});
