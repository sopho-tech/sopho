import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type {
  UsageByKindDto,
  UsageConversationQuestionsDto,
  UsageOverTimeDto,
  UsageSummaryDto,
} from "src/api/ai_usage";
import type {
  PlatformUsageConversationDto,
  PlatformUsagePage,
  PlatformUsageTenantDto,
  PlatformUsageUserDto,
} from "src/api/platform_ai_usage";
import {
  PlatformAIUsage,
  PlatformUsageByConversation,
  PlatformUsageByTenant,
  PlatformUsageByUser,
  PlatformUsageByWork,
  PlatformUsageConversationQuestions,
  PlatformUsageSummary,
} from "src/components/Platform/AIUsage";
import { PLATFORM_AI_OFF_NOTICE } from "src/components/Platform/AIUsage/constants";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import { CONVERSATION_ID } from "src/test/test_data";

const USAGE_TENANT_ID = "00000000-0000-4000-8000-000000000921";

const SUMMARY: UsageSummaryDto = {
  from: "2026-10-17",
  to: "2026-11-15",
  input_tokens: 60000,
  output_tokens: 12000,
  cached_input_tokens: 0,
  total_tokens: 72000,
  call_count: 95,
  has_usage: true,
  is_platform_ai_on: true,
  missing_token_counts_note: null,
};

const OVER_TIME: UsageOverTimeDto = {
  days: [
    {
      day: "2026-11-15",
      input_tokens: 60000,
      output_tokens: 12000,
      total_tokens: 72000,
      call_count: 95,
    },
  ],
};

const BY_KIND: UsageByKindDto = {
  kinds: [
    {
      kind: "ANSWERING_QUESTIONS",
      label: "Answering questions",
      total_tokens: 72000,
      call_count: 95,
    },
  ],
};

function usagePageOf<UsageRow>(
  usageRows: UsageRow[],
): PlatformUsagePage<UsageRow> {
  return {
    data: usageRows,
    total_pages: 1,
    page: 0,
    page_size: 10,
    total_items: usageRows.length,
  };
}

const BY_TENANT = usagePageOf<PlatformUsageTenantDto>([
  {
    tenant_id: USAGE_TENANT_ID,
    tenant_name: "Borealis Labs",
    total_tokens: 72000,
    call_count: 95,
    platform_ai_used_tokens: 88000,
    platform_ai_token_allowance: 100000,
  },
]);

const BY_USER = usagePageOf<PlatformUsageUserDto>([
  {
    user_id: "00000000-0000-4000-8000-000000000922",
    name: "Priya Nair",
    tenant_id: USAGE_TENANT_ID,
    tenant_name: "Borealis Labs",
    total_tokens: 72000,
    call_count: 95,
  },
]);

const BY_CONVERSATION = usagePageOf<PlatformUsageConversationDto>([
  {
    conversation_id: CONVERSATION_ID,
    tenant_id: USAGE_TENANT_ID,
    tenant_name: "Borealis Labs",
    name: "Weekly revenue by region",
    name_kind: "NAME",
    user_name: "Priya Nair",
    total_tokens: 72000,
    call_count: 95,
  },
]);

const CONVERSATION_QUESTIONS: UsageConversationQuestionsDto = {
  conversation_id: CONVERSATION_ID,
  name: "Weekly revenue by region",
  name_kind: "NAME",
  total_tokens: 72000,
  call_count: 95,
  question_count: 1,
  questions: [
    {
      conversation_message_id: "00000000-0000-4000-8000-000000000923",
      text: "What was revenue by region last week?",
      text_kind: "TEXT",
      first_call_at: "2026-11-15T09:12:00Z",
      total_tokens: 72000,
      call_count: 95,
    },
  ],
};

type RequestedQuestions = {
  tenantId: string;
  conversationId: string;
};

function platformUsageHandlers(
  summary: UsageSummaryDto,
  requestedSummaryQueries: string[],
  requestedQuestions: RequestedQuestions[],
) {
  return [
    http.get(API_ENDPOINTS.PLATFORM.AI_USAGE_SUMMARY, ({ request }) => {
      requestedSummaryQueries.push(new URL(request.url).search);
      return HttpResponse.json(summary);
    }),
    http.get(API_ENDPOINTS.PLATFORM.AI_USAGE_OVER_TIME, () =>
      HttpResponse.json(OVER_TIME),
    ),
    http.get(API_ENDPOINTS.PLATFORM.AI_USAGE_BY_KIND, () =>
      HttpResponse.json(BY_KIND),
    ),
    http.get(API_ENDPOINTS.PLATFORM.AI_USAGE_BY_TENANT, () =>
      HttpResponse.json(BY_TENANT),
    ),
    http.get(API_ENDPOINTS.PLATFORM.AI_USAGE_BY_USER, () =>
      HttpResponse.json(BY_USER),
    ),
    http.get(API_ENDPOINTS.PLATFORM.AI_USAGE_BY_CONVERSATION, () =>
      HttpResponse.json(BY_CONVERSATION),
    ),
    http.get(
      API_ENDPOINTS.PLATFORM.AI_USAGE_CONVERSATION_QUESTIONS,
      ({ params }) => {
        requestedQuestions.push({
          tenantId: String(params.tenant_id),
          conversationId: String(params.conversation_id),
        });
        return HttpResponse.json(CONVERSATION_QUESTIONS);
      },
    ),
  ];
}

const PLATFORM_USAGE_ROUTES = [
  {
    path: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE,
    element: <PlatformAIUsage />,
    children: [
      { path: "summary", element: <PlatformUsageSummary /> },
      { path: "by-tenant", element: <PlatformUsageByTenant /> },
      { path: "by-user", element: <PlatformUsageByUser /> },
      { path: "by-work", element: <PlatformUsageByWork /> },
      { path: "by-conversation", element: <PlatformUsageByConversation /> },
      {
        path: "by-conversation/:usageTenantId/:conversationId",
        element: <PlatformUsageConversationQuestions />,
      },
    ],
  },
];

async function renderPlatformUsagePage(usagePath: string) {
  return renderWithProviders({
    routes: PLATFORM_USAGE_ROUTES,
    initialPath: usagePath,
  });
}

describe("platform AI usage", () => {
  test("shows the totals across tenants and asks again when the range changes", async () => {
    const requestedSummaryQueries: string[] = [];
    worker.use(...platformUsageHandlers(SUMMARY, requestedSummaryQueries, []));
    const { router } = await renderPlatformUsagePage(
      APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_SUMMARY,
    );

    await expect
      .element(
        page.getByRole("group", { name: "Total tokens" }).getByText("72,000"),
      )
      .toBeVisible();
    await expect
      .element(page.getByRole("region", { name: "Tokens per day" }))
      .toBeVisible();
    await expect
      .element(page.getByText(PLATFORM_AI_OFF_NOTICE))
      .not.toBeInTheDocument();

    await page.getByRole("combobox", { name: "Date range" }).click();
    await page.getByRole("option", { name: "Last 7 days" }).click();

    await expect
      .poll(() => requestedSummaryQueries)
      .toEqual(["?last_day_count=30", "?last_day_count=7"]);
    expect(router.state.location.search).toBe("?days=7");
  });

  test("says platform AI is off when the backend says so", async () => {
    worker.use(
      ...platformUsageHandlers(
        { ...SUMMARY, is_platform_ai_on: false },
        [],
        [],
      ),
    );

    await renderPlatformUsagePage(APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_SUMMARY);

    await expect.element(page.getByText(PLATFORM_AI_OFF_NOTICE)).toBeVisible();
  });

  test("shows an empty state when no platform AI was used in the range", async () => {
    worker.use(
      ...platformUsageHandlers(
        { ...SUMMARY, total_tokens: 0, call_count: 0, has_usage: false },
        [],
        [],
      ),
    );

    await renderPlatformUsagePage(APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_SUMMARY);

    await expect
      .element(page.getByText("No platform AI usage in this range"))
      .toBeVisible();
    await expect
      .element(page.getByRole("group", { name: "Total tokens" }))
      .not.toBeInTheDocument();
  });

  test("shows each tenant with its tokens used against the allowance", async () => {
    worker.use(...platformUsageHandlers(SUMMARY, [], []));

    await renderPlatformUsagePage(
      APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_TENANT,
    );

    const tenantRow = page.getByRole("row", { name: "Borealis Labs" });
    await expect.element(tenantRow.getByText("72,000")).toBeVisible();
    await expect
      .element(tenantRow.getByText("88,000 of 100,000"))
      .toBeVisible();
    await expect
      .element(page.getByRole("tab", { name: "By Tenant" }))
      .toHaveAttribute("aria-selected", "true");
  });

  test("shows each user with their tenant", async () => {
    worker.use(...platformUsageHandlers(SUMMARY, [], []));

    await renderPlatformUsagePage(APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_USER);

    const userRow = page.getByRole("row", {
      name: "Priya Nair in Borealis Labs",
    });
    await expect.element(userRow.getByText("Borealis Labs")).toBeVisible();
    await expect.element(userRow.getByText("72,000")).toBeVisible();
  });

  test("starts a tab on its first page and keeps the range", async () => {
    worker.use(...platformUsageHandlers(SUMMARY, [], []));
    const { router } = await renderWithProviders({
      routes: PLATFORM_USAGE_ROUTES,
      initialPath: `${APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_USER}?days=7&page=3`,
    });

    await page.getByRole("tab", { name: "By Tenant" }).click();

    await expect
      .element(page.getByRole("row", { name: "Borealis Labs" }))
      .toBeVisible();
    expect(router.state.location.pathname).toBe(
      APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_TENANT,
    );
    expect(router.state.location.search).toBe("?days=7");
  });

  test("shows kinds of work as a pie chart", async () => {
    worker.use(...platformUsageHandlers(SUMMARY, [], []));

    await renderPlatformUsagePage(APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_WORK);

    await expect
      .element(
        page
          .getByRole("region", { name: "Tokens by kind of work" })
          .getByText("Answering questions"),
      )
      .toBeVisible();
  });

  test("opens a conversation's questions for its tenant and goes back", async () => {
    const requestedQuestions: RequestedQuestions[] = [];
    worker.use(...platformUsageHandlers(SUMMARY, [], requestedQuestions));
    const { router } = await renderPlatformUsagePage(
      APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_CONVERSATION,
    );

    await page
      .getByRole("row", { name: "Weekly revenue by region in Borealis Labs" })
      .click();

    await expect
      .element(page.getByText("What was revenue by region last week?"))
      .toBeVisible();
    expect(requestedQuestions).toEqual([
      { tenantId: USAGE_TENANT_ID, conversationId: CONVERSATION_ID },
    ]);
    expect(router.state.location.pathname).toBe(
      `${APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_CONVERSATION}/${USAGE_TENANT_ID}/${CONVERSATION_ID}`,
    );

    await page.getByRole("button", { name: "All conversations" }).click();

    await expect
      .element(
        page.getByRole("table", { name: "Platform AI usage by conversation" }),
      )
      .toBeVisible();
  });
});
