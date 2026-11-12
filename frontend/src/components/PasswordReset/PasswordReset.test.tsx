import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { isRouteErrorResponse, useRouteError } from "react-router";
import {
  ForgotPassword,
  ResetPassword,
  requirePasswordSignIn,
} from "src/components/PasswordReset";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";

function RouteErrorStatus() {
  const routeError = useRouteError();
  if (!isRouteErrorResponse(routeError)) {
    throw routeError;
  }
  return <p>{`Route error ${routeError.status}`}</p>;
}

const passwordResetRoutes = [
  {
    errorElement: <RouteErrorStatus />,
    children: [
      {
        path: APP_ROUTES.FORGOT_PASSWORD,
        loader: requirePasswordSignIn,
        element: <ForgotPassword />,
      },
      {
        path: APP_ROUTES.RESET_PASSWORD,
        loader: requirePasswordSignIn,
        element: <ResetPassword />,
      },
    ],
  },
];

function signInPageOptionsHandler(isPasswordSignInAvailable: boolean) {
  return http.get(API_ENDPOINTS.AUTH.SIGN_IN_PAGE_OPTIONS, () =>
    HttpResponse.json({
      providers: [],
      password_sign_in_available: isPasswordSignInAvailable,
      password_reset_available: isPasswordSignInAvailable,
    }),
  );
}

describe("password reset pages", () => {
  test("forgot password is not found when passwords are unavailable", async () => {
    worker.use(signInPageOptionsHandler(false));
    await renderWithProviders({
      routes: passwordResetRoutes,
      initialPath: APP_ROUTES.FORGOT_PASSWORD,
    });

    await expect.element(page.getByText("Route error 404")).toBeVisible();
    await expect
      .element(page.getByRole("heading", { name: "Forgot your password?" }))
      .not.toBeInTheDocument();
  });

  test("reset password is not found when passwords are unavailable", async () => {
    worker.use(signInPageOptionsHandler(false));
    await renderWithProviders({
      routes: passwordResetRoutes,
      initialPath: `${APP_ROUTES.RESET_PASSWORD}#token=any-token`,
    });

    await expect.element(page.getByText("Route error 404")).toBeVisible();
    await expect
      .element(page.getByRole("button", { name: /password/i }))
      .not.toBeInTheDocument();
  });

  test("forgot password opens when passwords are available", async () => {
    worker.use(signInPageOptionsHandler(true));
    await renderWithProviders({
      routes: passwordResetRoutes,
      initialPath: APP_ROUTES.FORGOT_PASSWORD,
    });

    await expect
      .element(page.getByRole("heading", { name: "Forgot your password?" }))
      .toBeVisible();
  });

  test("an address the app does not have gives the same not found", async () => {
    await renderWithProviders({
      routes: passwordResetRoutes,
      initialPath: "/no-such-page",
    });

    await expect.element(page.getByText("Route error 404")).toBeVisible();
  });
});
