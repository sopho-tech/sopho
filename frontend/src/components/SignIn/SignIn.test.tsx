import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import SignIn from "src/components/SignIn/SignIn";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { signedOutSessionHandler } from "src/test/msw/handlers";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import { TEST_USER } from "src/test/test_data";

const signInRoutes = [
  {
    element: <ProtectedRoute />,
    children: [{ path: APP_ROUTES.INDEX, element: <p>Home page</p> }],
  },
  { path: APP_ROUTES.SIGN_IN, element: <SignIn /> },
  { path: APP_ROUTES.FORGOT_PASSWORD, element: <p>Forgot password page</p> },
];

describe("sign in", () => {
  test("signs in with email and password and opens the app", async () => {
    let isSignedIn = false;
    let submittedCredentials: unknown = null;
    worker.use(
      http.get(API_ENDPOINTS.AUTH.GET_SESSION, () =>
        isSignedIn
          ? new HttpResponse(null, { status: 200 })
          : HttpResponse.json({ message: "Unauthorized" }, { status: 401 }),
      ),
      http.post(API_ENDPOINTS.AUTH.CREATE_SESSION, async ({ request }) => {
        submittedCredentials = await request.json();
        isSignedIn = true;
        return HttpResponse.json(TEST_USER);
      }),
    );
    const { router } = await renderWithProviders({
      routes: signInRoutes,
      initialPath: APP_ROUTES.SIGN_IN,
    });

    await page.getByLabelText("Email", { exact: true }).fill(TEST_USER.email);
    await page
      .getByLabelText("Password", { exact: true })
      .fill("correct-horse-battery");
    await page.getByRole("button", { name: "Sign in", exact: true }).click();

    await expect.element(page.getByText("Home page")).toBeVisible();
    expect(router.state.location.pathname).toBe(APP_ROUTES.INDEX);
    expect(submittedCredentials).toEqual({
      email: TEST_USER.email,
      password: "correct-horse-battery",
    });
  });

  test("asks for the email and the password before signing in", async () => {
    worker.use(signedOutSessionHandler);
    await renderWithProviders({
      routes: signInRoutes,
      initialPath: APP_ROUTES.SIGN_IN,
    });

    await page.getByRole("button", { name: "Sign in", exact: true }).click();

    await expect.element(page.getByText("Email is required")).toBeVisible();
    await expect.element(page.getByText("Password is required")).toBeVisible();
  });

  test("hides password reset until the backend says email is set up", async () => {
    worker.use(signedOutSessionHandler);
    await renderWithProviders({
      routes: signInRoutes,
      initialPath: APP_ROUTES.SIGN_IN,
    });

    await expect
      .element(page.getByRole("button", { name: "Sign in", exact: true }))
      .toBeVisible();
    await expect
      .element(page.getByRole("button", { name: "Forgot password?" }))
      .not.toBeInTheDocument();
  });

  test("opens password reset when email is set up", async () => {
    worker.use(
      signedOutSessionHandler,
      http.get(API_ENDPOINTS.AUTH.SIGN_IN_OPTIONS, () =>
        HttpResponse.json({ providers: [], password_reset_available: true }),
      ),
    );
    const { router } = await renderWithProviders({
      routes: signInRoutes,
      initialPath: APP_ROUTES.SIGN_IN,
    });

    await page.getByRole("button", { name: "Forgot password?" }).click();

    await expect.element(page.getByText("Forgot password page")).toBeVisible();
    expect(router.state.location.pathname).toBe(APP_ROUTES.FORGOT_PASSWORD);
  });
});
