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

    await page.getByPlaceholder("email").fill(TEST_USER.email);
    await page.getByPlaceholder("password").fill("correct-horse-battery");
    await page.getByRole("button", { name: "Sign In" }).click();

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

    await page.getByRole("button", { name: "Sign In" }).click();

    await expect.element(page.getByText("Email is required")).toBeVisible();
    await expect.element(page.getByText("Password is required")).toBeVisible();
  });
});
