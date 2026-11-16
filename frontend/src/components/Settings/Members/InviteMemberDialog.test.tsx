import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type { RoleDto } from "src/api/member/queries";
import { InviteMemberDialog } from "src/components/Settings/Members/InviteMemberDialog";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  tenantRoutePattern,
  testTenantApiUrl,
  testTenantPath,
} from "src/test/tenant_routes";

const MEMBER_ROLE: RoleDto = {
  name: "MEMBER",
  display_name: "Member",
  description: "Can create and edit their own work",
  is_system: true,
  assignable: true,
};

const MEMBER_LIMIT_REACHED_MESSAGE =
  "This tenant has reached its limit of 3 members";

function ignoreDialogEvent() {}

describe("invite member dialog", () => {
  test("shows the backend's message when the invitation is refused by the member limit", async () => {
    let requestedInvitation: unknown = null;
    worker.use(
      http.post(
        testTenantApiUrl(API_ENDPOINTS.INVITATION.TENANT),
        async ({ request }) => {
          requestedInvitation = await request.json();
          return HttpResponse.json(
            {
              error: MEMBER_LIMIT_REACHED_MESSAGE,
              error_code: "TENANT_MEMBER_LIMIT_REACHED",
            },
            { status: 409 },
          );
        },
      ),
    );
    await renderWithProviders({
      routes: [
        {
          path: tenantRoutePattern(APP_ROUTES.SETTINGS),
          element: (
            <InviteMemberDialog
              roles={[MEMBER_ROLE]}
              onInvited={ignoreDialogEvent}
              onClose={ignoreDialogEvent}
            />
          ),
        },
      ],
      initialPath: testTenantPath(APP_ROUTES.SETTINGS),
    });

    await page
      .getByRole("textbox", { name: "Email" })
      .fill("fourth@example.test");
    await page.getByRole("combobox", { name: "Role" }).click();
    await page.getByRole("option", { name: MEMBER_ROLE.display_name }).click();
    await page.getByRole("button", { name: "Send invitation" }).click();

    await expect
      .element(page.getByText(MEMBER_LIMIT_REACHED_MESSAGE))
      .toBeVisible();
    expect(requestedInvitation).toEqual({
      email: "fourth@example.test",
      role_name: MEMBER_ROLE.name,
    });
    await expect.element(page.getByRole("dialog")).toBeVisible();
  });
});
