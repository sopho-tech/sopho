import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page } from "vitest/browser";
import type {
  UsageByConversationDto,
  UsageByKindDto,
  UsageByUserDto,
  UsageConversationQuestionsDto,
  UsageOverTimeDto,
  UsageSummaryDto,
} from "src/api/ai_usage";
import {
  useMemberships,
  type TenantMembershipDto,
} from "src/api/tenant/queries";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { Settings } from "src/components/Settings";
import {
  AIUsage,
  UsageByConversation,
  UsageByUser,
  UsageByWork,
  UsageConversationQuestions,
  UsageSummary,
} from "src/components/Settings/AIUsage";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { FIRST_PAGE } from "src/constants/pagination";
import { PERMISSIONS } from "src/constants/permissions";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  tenantRoutePattern,
  testTenantApiUrl,
  testTenantPath,
} from "src/test/tenant_routes";
import { CONVERSATION_ID, TEST_TENANT_ID } from "src/test/test_data";

const PRIVATE_CONVERSATION_ID = "00000000-0000-4000-8000-000000000699";

const SUMMARY: UsageSummaryDto = {
  from: "2026-10-17",
  to: "2026-11-15",
  input_tokens: 180000,
  output_tokens: 28000,
  cached_input_tokens: 1000,
  total_tokens: 208000,
  call_count: 412,
  has_usage: true,
  is_platform_ai_on: false,
  missing_token_counts_note:
    "3 model calls in this range reported no token counts. They are included in the number of model calls but add no tokens.",
};

const EMPTY_SUMMARY: UsageSummaryDto = {
  ...SUMMARY,
  input_tokens: 0,
  output_tokens: 0,
  cached_input_tokens: 0,
  total_tokens: 0,
  call_count: 0,
  has_usage: false,
  missing_token_counts_note: null,
};

const OVER_TIME: UsageOverTimeDto = {
  days: [
    {
      day: "2026-11-14",
      input_tokens: 90000,
      output_tokens: 14000,
      total_tokens: 104000,
      call_count: 200,
    },
    {
      day: "2026-11-15",
      input_tokens: 90000,
      output_tokens: 14000,
      total_tokens: 104000,
      call_count: 212,
    },
  ],
};

const BY_USER: UsageByUserDto = {
  data: [
    {
      user_id: "00000000-0000-4000-8000-000000000001",
      name: "Priya Nair",
      total_tokens: 150000,
      call_count: 300,
    },
    { user_id: null, name: "Background", total_tokens: 58000, call_count: 112 },
  ],
  total_pages: 1,
  page: FIRST_PAGE,
  page_size: 10,
  total_items: 2,
};

const BY_KIND: UsageByKindDto = {
  kinds: [
    {
      kind: "ANSWERING_QUESTIONS",
      label: "Answering questions",
      total_tokens: 190000,
      call_count: 380,
    },
    {
      kind: "AI_KEY_TEST",
      label: "AI key test",
      total_tokens: 18000,
      call_count: 32,
    },
  ],
};

const BY_CONVERSATION: UsageByConversationDto = {
  data: [
    {
      conversation_id: CONVERSATION_ID,
      name: "Weekly revenue by region",
      name_kind: "NAME",
      user_name: "Priya Nair",
      total_tokens: 96420,
      call_count: 141,
    },
    {
      conversation_id: PRIVATE_CONVERSATION_ID,
      name: "Private conversation",
      name_kind: "PRIVATE",
      user_name: "Priya Nair",
      total_tokens: 4200,
      call_count: 12,
    },
  ],
  total_pages: 1,
  page: 0,
  page_size: 10,
  total_items: 2,
};

const CONVERSATION_QUESTIONS: UsageConversationQuestionsDto = {
  conversation_id: CONVERSATION_ID,
  name: "Weekly revenue by region",
  name_kind: "NAME",
  total_tokens: 96420,
  call_count: 141,
  question_count: 1,
  questions: [
    {
      conversation_message_id: "00000000-0000-4000-8000-000000000611",
      text: "What was revenue by region last week?",
      text_kind: "TEXT",
      first_call_at: "2026-11-14T09:12:00Z",
      total_tokens: 90000,
      call_count: 130,
    },
    {
      conversation_message_id: null,
      text: "Conversation naming and other model calls outside a question",
      text_kind: "NO_QUESTION",
      first_call_at: "2026-11-14T09:13:00Z",
      total_tokens: 6420,
      call_count: 11,
    },
  ],
};

const PRIVATE_CONVERSATION_QUESTIONS: UsageConversationQuestionsDto = {
  conversation_id: PRIVATE_CONVERSATION_ID,
  name: "Private conversation",
  name_kind: "PRIVATE",
  total_tokens: 4200,
  call_count: 12,
  question_count: 1,
  questions: [
    {
      conversation_message_id: "00000000-0000-4000-8000-000000000612",
      text: "Question in a private conversation",
      text_kind: "PRIVATE",
      first_call_at: "2026-11-13T10:00:00Z",
      total_tokens: 4200,
      call_count: 12,
    },
  ],
};

function membershipWith(permissions: string[]): TenantMembershipDto {
  return {
    tenant_id: TEST_TENANT_ID,
    tenant_name: "Acme Analytics",
    role_name: "OWNER",
    role_display_name: "Owner",
    permissions,
    suspended_message: null,
  };
}

function usageHandlers(
  summary: UsageSummaryDto,
  requestedSummaryQueries: string[],
) {
  return [
    http.get(
      testTenantApiUrl(API_ENDPOINTS.AI_USAGE.SUMMARY),
      ({ request }) => {
        requestedSummaryQueries.push(new URL(request.url).search);
        return HttpResponse.json(summary);
      },
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.AI_USAGE.OVER_TIME), () =>
      HttpResponse.json(OVER_TIME),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.AI_USAGE.BY_USER), () =>
      HttpResponse.json(BY_USER),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.AI_USAGE.BY_KIND), () =>
      HttpResponse.json(BY_KIND),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.AI_USAGE.BY_CONVERSATION), () =>
      HttpResponse.json(BY_CONVERSATION),
    ),
    http.get(
      testTenantApiUrl(API_ENDPOINTS.AI_USAGE.CONVERSATION_QUESTIONS),
      ({ params }) =>
        HttpResponse.json(
          params.conversation_id === PRIVATE_CONVERSATION_ID
            ? PRIVATE_CONVERSATION_QUESTIONS
            : CONVERSATION_QUESTIONS,
        ),
    ),
  ];
}

const USAGE_ROUTES = [
  {
    path: tenantRoutePattern(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE),
    element: <AIUsage />,
    children: [
      { path: "summary", element: <UsageSummary /> },
      { path: "by-user", element: <UsageByUser /> },
      { path: "by-work", element: <UsageByWork /> },
      { path: "by-conversation", element: <UsageByConversation /> },
      {
        path: "by-conversation/:conversationId",
        element: <UsageConversationQuestions />,
      },
    ],
  },
];

async function renderUsagePage(usagePath: string) {
  return renderWithProviders({
    routes: USAGE_ROUTES,
    initialPath: testTenantPath(usagePath),
  });
}

function SettingsOnceMembershipLoads() {
  const memberships = useMemberships();
  if (memberships.data === undefined) return null;
  return <Settings />;
}

async function renderSettingsWith(permissions: string[]) {
  worker.use(
    http.get(API_ENDPOINTS.TENANT.LIST, () =>
      HttpResponse.json([membershipWith(permissions)]),
    ),
  );
  await renderWithProviders({
    routes: [
      {
        element: <ProtectedRoute />,
        children: [
          {
            path: tenantRoutePattern(APP_ROUTES.SETTINGS),
            element: <SettingsOnceMembershipLoads />,
          },
        ],
      },
    ],
    initialPath: testTenantPath(APP_ROUTES.SETTINGS),
  });
}

describe("AI usage", () => {
  test("shows the totals and the tokens per day on the summary tab", async () => {
    worker.use(...usageHandlers(SUMMARY, []));

    await renderUsagePage(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_SUMMARY);

    await expect
      .element(
        page.getByRole("group", { name: "Total tokens" }).getByText("208,000"),
      )
      .toBeVisible();
    await expect
      .element(
        page.getByRole("group", { name: "Model calls" }).getByText("412"),
      )
      .toBeVisible();
    await expect
      .element(
        page
          .getByRole("group", { name: "Total tokens" })
          .getByText("Used in the time range"),
      )
      .toBeVisible();
    await expect
      .element(page.getByText("Oct 17, 2026 to Nov 15, 2026"))
      .toBeVisible();
    await expect
      .element(page.getByRole("region", { name: "Tokens per day" }))
      .toBeVisible();
    await expect
      .element(page.getByText(SUMMARY.missing_token_counts_note as string))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByRole("tab", { name: "Summary" }))
      .toHaveAttribute("aria-selected", "true");
  });

  test("asks the backend again and keeps the range in the address when the range changes", async () => {
    const requestedSummaryQueries: string[] = [];
    worker.use(...usageHandlers(SUMMARY, requestedSummaryQueries));
    const { router } = await renderUsagePage(
      APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_SUMMARY,
    );
    await expect
      .element(page.getByRole("group", { name: "Total tokens" }))
      .toBeVisible();

    await page.getByRole("combobox", { name: "Date range" }).click();
    await page.getByRole("option", { name: "Last 7 days" }).click();

    await expect
      .poll(() => requestedSummaryQueries)
      .toEqual(["?last_day_count=30", "?last_day_count=7"]);
    expect(router.state.location.search).toBe("?days=7");

    await page.getByRole("tab", { name: "By Work" }).click();

    await expect
      .element(page.getByRole("region", { name: "Tokens by kind of work" }))
      .toBeVisible();
    expect(router.state.location.pathname).toBe(
      testTenantPath(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_WORK),
    );
    expect(router.state.location.search).toBe("?days=7");
  });

  test("offers the AI key choice only when the backend says platform AI is on", async () => {
    worker.use(...usageHandlers(SUMMARY, []));

    await renderUsagePage(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_SUMMARY);

    await expect
      .element(page.getByRole("group", { name: "Total tokens" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("combobox", { name: "AI key" }))
      .not.toBeInTheDocument();
  });

  test("asks the backend for one key's usage and keeps the key in the address", async () => {
    const requestedSummaryQueries: string[] = [];
    worker.use(
      ...usageHandlers(
        { ...SUMMARY, is_platform_ai_on: true },
        requestedSummaryQueries,
      ),
    );
    const { router } = await renderUsagePage(
      APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_SUMMARY,
    );

    await page.getByRole("combobox", { name: "AI key" }).click();
    await page.getByRole("option", { name: "Platform AI" }).click();

    await expect
      .poll(() => requestedSummaryQueries)
      .toEqual([
        "?last_day_count=30",
        "?last_day_count=30&key_source=PLATFORM",
      ]);
    expect(router.state.location.search).toBe("?key=PLATFORM");

    await page.getByRole("combobox", { name: "AI key" }).click();
    await page.getByRole("option", { name: "All usage" }).click();

    await expect
      .poll(() => requestedSummaryQueries.length)
      .toBeGreaterThanOrEqual(2);
    expect(router.state.location.search).toBe("");
  });

  test("lets a key the backend refuses be cleared from the address", async () => {
    worker.use(
      http.get(
        testTenantApiUrl(API_ENDPOINTS.AI_USAGE.SUMMARY),
        ({ request }) =>
          new URL(request.url).searchParams.has("key_source")
            ? HttpResponse.json(
                { error: "unknown variant `SOMEONE`" },
                { status: 400 },
              )
            : HttpResponse.json(SUMMARY),
      ),
      ...usageHandlers(SUMMARY, []),
    );
    const { router } = await renderUsagePage(
      `${APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_SUMMARY}?key=SOMEONE`,
    );

    await expect
      .element(page.getByText("unknown variant `SOMEONE`"))
      .toBeVisible();
    await page.getByRole("combobox", { name: "AI key" }).click();
    await page.getByRole("option", { name: "All usage" }).click();

    await expect
      .element(page.getByRole("group", { name: "Total tokens" }))
      .toBeVisible();
    expect(router.state.location.search).toBe("");
  });

  test("shows users as a table on the address of the user tab", async () => {
    worker.use(...usageHandlers(SUMMARY, []));

    await renderUsagePage(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_USER);

    await expect
      .element(page.getByRole("table", { name: "Usage by user" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("region", { name: "Tokens by user" }))
      .not.toBeInTheDocument();
    await expect
      .element(
        page.getByRole("row", { name: "Background" }).getByText("58,000"),
      )
      .toBeVisible();
    await expect
      .element(page.getByRole("tab", { name: "By User" }))
      .toHaveAttribute("aria-selected", "true");
  });

  test("shows kinds of work as a pie chart only on the address of the work tab", async () => {
    worker.use(...usageHandlers(SUMMARY, []));

    await renderUsagePage(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_WORK);

    await expect
      .element(page.getByRole("region", { name: "Tokens by kind of work" }))
      .toBeVisible();
    await expect
      .element(
        page
          .getByRole("region", { name: "Tokens by kind of work" })
          .getByText("AI key test"),
      )
      .toBeVisible();
    await expect.element(page.getByRole("table")).not.toBeInTheDocument();
  });

  test("opens the questions of a conversation on their own address when its row is chosen", async () => {
    worker.use(...usageHandlers(SUMMARY, []));
    const { router } = await renderUsagePage(
      APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_CONVERSATION,
    );
    await expect
      .element(page.getByRole("table", { name: "Usage by conversation" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("region", { name: "Tokens by conversation" }))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: "All conversations" }))
      .not.toBeInTheDocument();

    await page.getByRole("row", { name: "Weekly revenue by region" }).click();

    const questions = page.getByRole("table", {
      name: "Questions of Weekly revenue by region",
    });
    await expect
      .element(
        questions
          .getByRole("row", { name: "What was revenue by region last week?" })
          .getByText("90,000"),
      )
      .toBeVisible();
    await expect
      .element(questions.getByText("Nov 14, 09:12 UTC"))
      .toBeVisible();
    await expect
      .element(
        questions.getByText(
          "Conversation naming and other model calls outside a question",
        ),
      )
      .toBeVisible();
    expect(router.state.location.pathname).toBe(
      testTenantPath(
        `${APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_CONVERSATION}/${CONVERSATION_ID}`,
      ),
    );
    await expect.element(page.getByRole("dialog")).not.toBeInTheDocument();

    await page.getByRole("button", { name: "All conversations" }).click();

    await expect
      .element(page.getByRole("table", { name: "Usage by conversation" }))
      .toBeVisible();
  });

  test("searching users asks the backend for the matching users and says when none match", async () => {
    const requestedSearches: (string | null)[] = [];
    worker.use(
      http.get(
        testTenantApiUrl(API_ENDPOINTS.AI_USAGE.BY_USER),
        ({ request }) => {
          const requestedSearch = new URL(request.url).searchParams.get(
            "search",
          );
          requestedSearches.push(requestedSearch);
          return HttpResponse.json(
            requestedSearch === null
              ? BY_USER
              : { ...BY_USER, data: [], total_pages: 0, total_items: 0 },
          );
        },
      ),
      ...usageHandlers(SUMMARY, []),
    );
    const { router } = await renderUsagePage(
      APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_USER,
    );
    await expect
      .element(page.getByRole("row", { name: "Priya Nair" }))
      .toBeVisible();

    await page.getByRole("textbox", { name: "Search users" }).fill("nobody");

    await expect
      .element(page.getByText("No entries match your search."))
      .toBeVisible();
    expect(requestedSearches).toContain("nobody");
    expect(router.state.location.search).toBe("?search=nobody");
  });

  test("sorting conversations by a column asks the backend for that order and flips it on a second click", async () => {
    const requestedOrders: string[] = [];
    worker.use(
      http.get(
        testTenantApiUrl(API_ENDPOINTS.AI_USAGE.BY_CONVERSATION),
        ({ request }) => {
          const requestedSearchParams = new URL(request.url).searchParams;
          requestedOrders.push(
            `${requestedSearchParams.get("sort_by")} ${requestedSearchParams.get("sort_direction")}`,
          );
          return HttpResponse.json(BY_CONVERSATION);
        },
      ),
      ...usageHandlers(SUMMARY, []),
    );
    await renderUsagePage(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_CONVERSATION);
    await expect
      .element(page.getByRole("columnheader", { name: "Tokens" }))
      .toHaveAttribute("aria-sort", "descending");

    await page.getByRole("button", { name: "Model calls" }).click();
    await expect
      .element(page.getByRole("columnheader", { name: "Model calls" }))
      .toHaveAttribute("aria-sort", "descending");
    await page.getByRole("button", { name: "Model calls" }).click();

    await expect
      .element(page.getByRole("columnheader", { name: "Model calls" }))
      .toHaveAttribute("aria-sort", "ascending");
    await expect
      .element(page.getByRole("columnheader", { name: "Tokens" }))
      .toHaveAttribute("aria-sort", "none");
    await expect
      .poll(() => requestedOrders)
      .toEqual(["total_tokens desc", "call_count desc", "call_count asc"]);
  });

  test("moving to the next page of conversations asks the backend for that page and shows its rows", async () => {
    const secondPageNumber = FIRST_PAGE + 1;
    const requestedPages: (string | null)[] = [];
    worker.use(
      http.get(
        testTenantApiUrl(API_ENDPOINTS.AI_USAGE.BY_CONVERSATION),
        ({ request }) => {
          const requestedPage = new URL(request.url).searchParams.get("page");
          requestedPages.push(requestedPage);
          return HttpResponse.json(
            requestedPage === String(secondPageNumber)
              ? {
                  ...BY_CONVERSATION,
                  data: [BY_CONVERSATION.data[1]],
                  total_pages: 2,
                  page: secondPageNumber,
                }
              : {
                  ...BY_CONVERSATION,
                  data: [BY_CONVERSATION.data[0]],
                  total_pages: 2,
                  page: FIRST_PAGE,
                },
          );
        },
      ),
      ...usageHandlers(SUMMARY, []),
    );
    const { router } = await renderUsagePage(
      APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_CONVERSATION,
    );
    await expect
      .element(page.getByRole("row", { name: "Weekly revenue by region" }))
      .toBeVisible();
    expect(requestedPages).not.toContain(String(secondPageNumber));

    await page
      .getByRole("button", { name: String(secondPageNumber + 1), exact: true })
      .click();

    await expect
      .element(page.getByRole("row", { name: "Private conversation" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("row", { name: "Weekly revenue by region" }))
      .not.toBeInTheDocument();
    expect(requestedPages[requestedPages.length - 1]).toBe(
      String(secondPageNumber),
    );
    expect(new URLSearchParams(router.state.location.search).get("page")).toBe(
      String(secondPageNumber),
    );
  });

  test("shows the backend's label for a private conversation and its questions", async () => {
    worker.use(...usageHandlers(SUMMARY, []));

    await renderUsagePage(
      `${APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_CONVERSATION}/${PRIVATE_CONVERSATION_ID}`,
    );

    await expect
      .element(
        page
          .getByRole("table", { name: "Questions of Private conversation" })
          .getByText("Question in a private conversation"),
      )
      .toBeVisible();
  });

  test("explains what will appear when there is no usage in the range", async () => {
    worker.use(...usageHandlers(EMPTY_SUMMARY, []));

    await renderUsagePage(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_SUMMARY);

    await expect
      .element(page.getByRole("heading", { name: "No AI usage in this range" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("group", { name: "Total tokens" }))
      .not.toBeInTheDocument();
  });

  test("shows the backend's message when usage cannot be loaded", async () => {
    worker.use(
      http.get(testTenantApiUrl(API_ENDPOINTS.AI_USAGE.SUMMARY), () =>
        HttpResponse.json(
          { error: "Permission denied: usage:read" },
          { status: 403 },
        ),
      ),
      ...usageHandlers(SUMMARY, []),
    );

    await renderUsagePage(APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_SUMMARY);

    await expect
      .element(page.getByText("Permission denied: usage:read"))
      .toBeVisible();
  });
});

describe("AI usage entry in Settings", () => {
  test("is shown to a user who may read usage", async () => {
    await renderSettingsWith([PERMISSIONS.USAGE_READ]);

    await expect
      .element(page.getByRole("link", { name: "AI Usage" }))
      .toBeVisible();
  });

  test("is hidden from a user who may not read usage", async () => {
    await renderSettingsWith([]);

    await expect
      .element(page.getByRole("link", { name: "AI Configurations" }))
      .toBeVisible();
    await expect
      .element(page.getByRole("link", { name: "AI Usage" }))
      .not.toBeInTheDocument();
  });
});
