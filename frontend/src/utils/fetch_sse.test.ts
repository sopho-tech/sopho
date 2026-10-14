import { http, HttpResponse } from "msw";
import { describe, expect, test } from "vitest";
import { server } from "src/test/msw/node_server";
import { streamSse } from "src/utils/fetch_sse";

const STREAM_URL = "http://localhost/api/v1/conversational_analytics/stream";

function eventStreamResponse(chunks: string[]) {
  const encoder = new TextEncoder();
  const body = new ReadableStream<Uint8Array>({
    start(controller) {
      for (const chunk of chunks) {
        controller.enqueue(encoder.encode(chunk));
      }
      controller.close();
    },
  });
  return new HttpResponse(body, {
    headers: { "Content-Type": "text/event-stream" },
  });
}

async function collectMessages(chunks: string[]) {
  server.use(http.get(STREAM_URL, () => eventStreamResponse(chunks)));
  const messages: string[] = [];
  await streamSse(STREAM_URL, {}, { onMessage: (data) => messages.push(data) });
  return messages;
}

describe("streamSse", () => {
  test("delivers events that are split across network chunks", async () => {
    const messages = await collectMessages([
      'data: {"event_name":"star',
      'ting"}\n\ndata: {"event_name":"completed"}\n\n',
    ]);

    expect(messages).toEqual([
      '{"event_name":"starting"}',
      '{"event_name":"completed"}',
    ]);
  });

  test("joins multi-line data, skips comments and accepts CRLF line endings", async () => {
    const messages = await collectMessages([
      ": keep-alive\r\n\r\n",
      "data: first line\r\ndata: second line\r\n\r\n",
    ]);

    expect(messages).toEqual(["first line\nsecond line"]);
  });

  test("delivers a final event that the server closes without a blank line", async () => {
    const messages = await collectMessages(["data: one\n\n", "data: two"]);

    expect(messages).toEqual(["one", "two"]);
  });

  test("rejects with the server's error message when the response fails", async () => {
    server.use(
      http.get(STREAM_URL, () =>
        HttpResponse.json({ error: "AI is not configured" }, { status: 403 }),
      ),
    );

    await expect(
      streamSse(STREAM_URL, {}, { onMessage: () => undefined }),
    ).rejects.toThrow("AI is not configured");
  });

  test("returns without delivering events when the request is aborted", async () => {
    server.use(
      http.get(STREAM_URL, () => eventStreamResponse(["data: late\n\n"])),
    );
    const abortController = new AbortController();
    abortController.abort();
    const messages: string[] = [];

    await streamSse(
      STREAM_URL,
      { signal: abortController.signal },
      { onMessage: (data) => messages.push(data) },
    );

    expect(messages).toEqual([]);
  });
});
