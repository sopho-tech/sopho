import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { page, userEvent } from "vitest/browser";
import { ConversationalAnalytics } from "src/components/ConversationalAnalytics";
import { ExistingConversationPanel } from "src/components/ConversationalAnalytics/ExistingConversationPanel";
import { NewConversationPanel } from "src/components/ConversationalAnalytics/NewConversationPanel";
import { AgentEventName } from "src/components/ConversationalAnalytics/dto";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { worker } from "src/test/msw/browser_worker";
import { renderWithProviders } from "src/test/render_with_providers";
import {
  CONVERSATION,
  CONVERSATION_ID,
  ORDER_STATUS_ANSWER_EVENTS,
  ORDER_STATUS_NARRATION,
  ORDER_STATUS_QUESTION,
  SQLITE_CONNECTION,
  conversationWithMessages,
} from "src/test/test_data";

const conversationRoutes = [
  {
    path: APP_ROUTES.CONVERSATIONAL_ANALYTICS,
    element: <ConversationalAnalytics />,
    children: [
      {
        path: APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.INDEX,
        element: <NewConversationPanel />,
      },
      {
        path: APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION,
        element: <ExistingConversationPanel />,
      },
    ],
  },
];

function serverSentEvent(event: object) {
  return new TextEncoder().encode(`data: ${JSON.stringify(event)}\n\n`);
}

describe("conversational analytics", () => {
  test("a question sent from the composer streams back the answer", async () => {
    let releaseCompletion: () => void = () => {
      throw new Error("the completion stream has not started");
    };
    const completionReleased = new Promise<void>((resolve) => {
      releaseCompletion = resolve;
    });
    let isAnswered = false;
    let createdConversation: unknown = null;
    worker.use(
      http.get(API_ENDPOINTS.AI_CONFIGURATION.GET, () =>
        HttpResponse.json({ status: "live", provider: "anthropic" }),
      ),
      http.get(API_ENDPOINTS.CONNECTION.GET_ALL, () =>
        HttpResponse.json([SQLITE_CONNECTION]),
      ),
      http.get(API_ENDPOINTS.SUGGESTED_QUESTION.GET_BY_CONNECTION, () =>
        HttpResponse.json([]),
      ),
      http.get(API_ENDPOINTS.SEARCH.SEARCH, () =>
        HttpResponse.json({ data: [] }),
      ),
      http.get(API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.GET_ALL, () =>
        HttpResponse.json({ items: [], total: 0, page: 1, page_size: 25 }),
      ),
      http.post(
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.CREATE,
        async ({ request }) => {
          createdConversation = await request.json();
          return HttpResponse.json(CONVERSATION, { status: 201 });
        },
      ),
      http.get(API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.GET_BY_ID, () =>
        HttpResponse.json(
          conversationWithMessages(
            isAnswered
              ? [
                  ...ORDER_STATUS_ANSWER_EVENTS,
                  { event_name: AgentEventName.Completed },
                ]
              : null,
          ),
        ),
      ),
      http.post(API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.SUGGEST_NAME, () =>
        HttpResponse.json({ ...CONVERSATION, name: "Orders by status" }),
      ),
      http.post(API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.COMPLETION, () => {
        const body = new ReadableStream<Uint8Array>({
          async start(controller) {
            for (const event of ORDER_STATUS_ANSWER_EVENTS) {
              controller.enqueue(serverSentEvent(event));
            }
            await completionReleased;
            isAnswered = true;
            controller.enqueue(
              serverSentEvent({ event_name: AgentEventName.Completed }),
            );
            controller.close();
          },
        });
        return new HttpResponse(body, {
          headers: { "Content-Type": "text/event-stream" },
        });
      }),
    );
    const { router } = await renderWithProviders({
      routes: conversationRoutes,
      initialPath: APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.INDEX,
    });

    await page
      .getByRole("textbox", { name: "Ask anything about your data" })
      .click();
    await userEvent.keyboard(ORDER_STATUS_QUESTION);
    await userEvent.keyboard("{Enter}");

    await expect.element(page.getByText(ORDER_STATUS_NARRATION)).toBeVisible();
    expect(router.state.location.pathname).toBe(
      APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION.replace(
        ":id",
        CONVERSATION_ID,
      ),
    );
    expect(createdConversation).toEqual({
      connection_id: SQLITE_CONNECTION.id,
      segments: [{ type: "TEXT", text: ORDER_STATUS_QUESTION }],
    });
    releaseCompletion();
    await expect.poll(() => isAnswered).toBe(true);
    await expect.element(page.getByText(ORDER_STATUS_NARRATION)).toBeVisible();
    await expect
      .element(page.getByText(ORDER_STATUS_QUESTION, { exact: true }))
      .toBeVisible();
  });
});
