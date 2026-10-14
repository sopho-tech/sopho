import "src/css/index.css";
import { afterAll, afterEach, beforeAll } from "vitest";
import { worker } from "src/test/msw/browser_worker";
import {
  assertEveryRequestWasHandled,
  recordUnhandledRequest,
} from "src/test/msw/unhandled_requests";

beforeAll(async () => {
  await worker.start({ quiet: true, onUnhandledFrame: recordUnhandledRequest });
});

afterEach(() => {
  worker.resetHandlers();
  assertEveryRequestWasHandled();
});

afterAll(() => {
  void worker.stop();
});
