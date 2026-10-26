import { http, HttpResponse } from "msw";
import { afterEach, describe, expect, test } from "vitest";
import { page } from "vitest/browser";
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
} from "src/utils/pending_invitation";

const DEAD_INVITATION_TOKEN = "dead-invitation-token";
const DEAD_INVITATION_MESSAGE =
  "This invitation is no longer valid. Ask the person who invited you for a new one";

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
});
