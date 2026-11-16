import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type { AiConfiguration } from "src/api/ai_configuration";
import { AIConfiguration as AIConfigurationPage } from "src/components/Settings/AIConfiguration";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  tenantRoutePattern,
  testTenantApiUrl,
  testTenantPath,
} from "src/test/tenant_routes";
import {
  AI_NOT_CONFIGURED,
  AI_ON_OWN_KEY,
  AI_ON_THE_PLATFORM_KEY,
  AI_WITH_THE_ALLOWANCE_USED,
} from "src/test/test_data";

async function renderAiConfigurations(aiConfiguration: AiConfiguration) {
  worker.use(
    http.get(testTenantApiUrl(API_ENDPOINTS.AI_CONFIGURATION.GET), () =>
      HttpResponse.json(aiConfiguration),
    ),
  );
  await renderWithProviders({
    routes: [
      {
        path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.AI_CONFIGURATIONS),
        element: <AIConfigurationPage />,
      },
    ],
    initialPath: testTenantPath(APP_ROUTES.SETTINGS_ROUTES.AI_CONFIGURATIONS),
  });
}

describe("AI configurations", () => {
  test("shows the platform AI allowance the backend returns to a tenant with no key", async () => {
    await renderAiConfigurations(AI_ON_THE_PLATFORM_KEY);

    const platformAiCard = page.getByRole("region", { name: "Platform AI" });
    await expect
      .element(platformAiCard.getByText("41,250 of 100,000"))
      .toBeVisible();
    await expect.element(platformAiCard.getByText("58,750")).toBeVisible();
    await expect
      .element(platformAiCard.getByRole("meter", { name: "Allowance usage" }))
      .toHaveAttribute("aria-valuenow", "41");
    await expect.element(platformAiCard.getByText("41%")).toBeVisible();
  });

  test("shows the backend's message when the platform AI allowance is used", async () => {
    await renderAiConfigurations(AI_WITH_THE_ALLOWANCE_USED);

    const platformAiCard = page.getByRole("region", { name: "Platform AI" });
    await expect
      .element(platformAiCard.getByText("104,300 of 100,000"))
      .toBeVisible();
    await expect
      .element(platformAiCard.getByRole("meter", { name: "Allowance usage" }))
      .toHaveAttribute("aria-valuenow", "100");
    await expect.element(platformAiCard.getByText("104%")).toBeVisible();
    await expect
      .element(
        platformAiCard.getByText(
          "This tenant has used its platform AI allowance. Add your own AI key in Settings to keep using AI.",
        ),
      )
      .toBeVisible();
  });

  test("shows no platform AI card to a tenant on its own key", async () => {
    await renderAiConfigurations(AI_ON_OWN_KEY);

    await expect
      .element(page.getByRole("row", { name: "Anthropic" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("region", { name: "Platform AI" }))
      .not.toBeInTheDocument();
  });

  test("asks for a provider when platform AI is off and no key is saved", async () => {
    await renderAiConfigurations(AI_NOT_CONFIGURED);

    await expect
      .element(
        page.getByText(
          "No provider is set up yet. Add one below to turn on AI features.",
        ),
      )
      .toBeVisible();
    await expect
      .element(page.getByRole("region", { name: "Platform AI" }))
      .not.toBeInTheDocument();
  });
});
