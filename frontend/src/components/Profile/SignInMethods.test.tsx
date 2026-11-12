import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type { MySignInMethodsDto } from "src/api/auth/queries";
import { SignInMethods } from "src/components/Profile/SignInMethods";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";

const signInMethodsRoutes = [
  { path: APP_ROUTES.PROFILE, element: <SignInMethods /> },
];

function mySignInMethods(isPasswordAvailable: boolean): MySignInMethodsDto {
  return {
    has_password: true,
    password_available: isPasswordAvailable,
    providers: [
      {
        name: "google",
        display_name: "Google",
        connected: true,
        email: "member@example.test",
        can_disconnect: false,
        disconnect_blocked_reason:
          "This is your only way to sign in. Connect another sign-in method before you disconnect it.",
      },
    ],
  };
}

describe("profile sign-in methods", () => {
  test("hides the password row when passwords are unavailable", async () => {
    worker.use(
      http.get(API_ENDPOINTS.AUTH.MY_SIGN_IN_METHODS, () =>
        HttpResponse.json(mySignInMethods(false)),
      ),
    );
    await renderWithProviders({
      routes: signInMethodsRoutes,
      initialPath: APP_ROUTES.PROFILE,
    });

    await expect
      .element(page.getByRole("row", { name: "Google" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("row", { name: "Password" }))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: "Change password" }))
      .not.toBeInTheDocument();
  });

  test("shows the password row when passwords are available", async () => {
    worker.use(
      http.get(API_ENDPOINTS.AUTH.MY_SIGN_IN_METHODS, () =>
        HttpResponse.json(mySignInMethods(true)),
      ),
    );
    await renderWithProviders({
      routes: signInMethodsRoutes,
      initialPath: APP_ROUTES.PROFILE,
    });

    await expect
      .element(page.getByRole("row", { name: "Password" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("button", { name: "Change password" }))
      .toBeVisible();
  });
});
