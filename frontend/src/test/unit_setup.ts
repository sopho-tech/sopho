import { afterAll, afterEach, beforeAll } from "vitest";
import { server } from "src/test/msw/node_server";
import {
  assertEveryRequestWasHandled,
  recordUnhandledRequest,
} from "src/test/msw/unhandled_requests";

beforeAll(() => {
  server.listen({ onUnhandledFrame: recordUnhandledRequest });
});

afterEach(() => {
  server.resetHandlers();
  assertEveryRequestWasHandled();
});

afterAll(() => {
  assertEveryRequestWasHandled();
  server.close();
});
