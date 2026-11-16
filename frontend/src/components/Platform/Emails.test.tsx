import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type {
  EmailLogPageDto,
  EmailTemplatePreviewDto,
} from "src/api/platform/queries";
import { EmailTemplates } from "src/components/Platform/EmailTemplates";
import { SentEmails } from "src/components/Platform/SentEmails";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";
import { worker } from "src/test/msw/browser_worker";
import { signedInSessionHandler } from "src/test/msw/handlers";
import { renderWithProviders } from "src/test/render_with_providers";
import { TEST_USER } from "src/test/test_data";

const PASSWORD_RESET_TEMPLATE_PATH = `${APP_ROUTES.PLATFORM_ROUTES.EMAIL_TEMPLATES}?kind=PASSWORD_RESET`;

const EMAIL_LOG_PAGE: EmailLogPageDto = {
  emails: [
    {
      id: "00000000-0000-4000-8000-000000000921",
      kind: "PASSWORD_RESET",
      kind_display_name: "Password reset",
      recipient: "ada@example.com",
      subject: "Reset your password",
      status: "SENT",
      attempt_count: 1,
      last_error: null,
      created_at: "2026-10-01T09:00:00Z",
      next_attempt_at: "2026-10-01T09:00:00Z",
      sent_at: "2026-10-01T09:00:05Z",
      can_retry: false,
    },
  ],
  page: 0,
  page_size: 10,
  total: 1,
  total_pages: 1,
};

const EMAIL_TEMPLATES: EmailTemplatePreviewDto[] = [
  {
    kind: "INVITATION",
    kind_display_name: "Invitation",
    sent_when: "Someone is invited to a tenant",
    subject: "You are invited",
    html_body: "<p>Invitation body</p>",
    text_body: "Invitation body",
  },
  {
    kind: "PASSWORD_RESET",
    kind_display_name: "Password reset",
    sent_when: "Someone asks to reset their password",
    subject: "Reset your password",
    html_body: "<p>Password reset body</p>",
    text_body: "Password reset body",
  },
];

const emailRoutes = [
  {
    element: <ProtectedRoute />,
    children: [
      {
        path: APP_ROUTES.PLATFORM_ROUTES.SENT_EMAILS,
        element: <SentEmails />,
      },
      {
        path: APP_ROUTES.PLATFORM_ROUTES.EMAIL_TEMPLATES,
        element: <EmailTemplates />,
      },
    ],
  },
];

const emailHandlers = [
  signedInSessionHandler,
  http.get(API_ENDPOINTS.AUTH.GET_PROFILE_DETAILS, () =>
    HttpResponse.json({
      ...TEST_USER,
      platform_permissions: [PLATFORM_PERMISSIONS.EMAIL_MANAGE],
    }),
  ),
  http.get(API_ENDPOINTS.PLATFORM.EMAILS, () =>
    HttpResponse.json(EMAIL_LOG_PAGE),
  ),
  http.get(API_ENDPOINTS.PLATFORM.EMAIL_TEMPLATES, () =>
    HttpResponse.json(EMAIL_TEMPLATES),
  ),
];

describe("platform emails", () => {
  test("links a sent email's type to its template", async () => {
    worker.use(...emailHandlers);
    const { router } = await renderWithProviders({
      routes: emailRoutes,
      initialPath: APP_ROUTES.PLATFORM_ROUTES.SENT_EMAILS,
    });

    const templateLink = page.getByRole("link", { name: "Password reset" });
    await expect
      .element(templateLink)
      .toHaveAttribute("href", PASSWORD_RESET_TEMPLATE_PATH);
    await templateLink.click();

    await expect
      .element(page.getByRole("heading", { name: "Reset your password" }))
      .toBeVisible();
    expect(
      `${router.state.location.pathname}${router.state.location.search}`,
    ).toBe(PASSWORD_RESET_TEMPLATE_PATH);
    await expect
      .element(page.getByRole("button", { name: "Password reset" }))
      .toHaveAttribute("aria-current", "true");
  });

  test("opens the first template and keeps the chosen one in the address", async () => {
    worker.use(...emailHandlers);
    const { router } = await renderWithProviders({
      routes: emailRoutes,
      initialPath: APP_ROUTES.PLATFORM_ROUTES.EMAIL_TEMPLATES,
    });

    await expect
      .element(page.getByRole("heading", { name: "You are invited" }))
      .toBeVisible();
    await page.getByRole("button", { name: "Password reset" }).click();
    await expect
      .element(page.getByRole("heading", { name: "Reset your password" }))
      .toBeVisible();
    expect(router.state.location.search).toBe("?kind=PASSWORD_RESET");
  });
});
