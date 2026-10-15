import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import SignIn from "src/components/SignIn/SignIn";
import { APP_ROUTES } from "src/constants/app_routes";
import {
  signedInSessionHandler,
  signedOutSessionHandler,
} from "src/test/msw/handlers";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";

const protectedRoutes = [
  {
    element: <ProtectedRoute />,
    children: [{ path: APP_ROUTES.CANVASES, element: <p>Canvases page</p> }],
  },
  { path: APP_ROUTES.SIGN_IN, element: <SignIn /> },
];

describe("protected routes", () => {
  test("send a visitor without a session to the sign-in page", async () => {
    worker.use(signedOutSessionHandler);

    const { router } = await renderWithProviders({
      routes: protectedRoutes,
      initialPath: APP_ROUTES.CANVASES,
    });

    await expect
      .element(page.getByRole("button", { name: "Sign In" }))
      .toBeVisible();
    expect(router.state.location.pathname).toBe(APP_ROUTES.SIGN_IN);
    await expect
      .element(page.getByText("Canvases page"))
      .not.toBeInTheDocument();
  });

  test("show the page to a signed-in user", async () => {
    worker.use(signedInSessionHandler);

    const { router } = await renderWithProviders({
      routes: protectedRoutes,
      initialPath: APP_ROUTES.CANVASES,
    });

    await expect.element(page.getByText("Canvases page")).toBeVisible();
    expect(router.state.location.pathname).toBe(APP_ROUTES.CANVASES);
  });
});
