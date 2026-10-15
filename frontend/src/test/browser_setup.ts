import "src/css/index.css";
import { afterAll, afterEach, beforeAll } from "vitest";
import { cleanup } from "vitest-browser-react";
import { worker } from "src/test/msw/browser_worker";
import {
  assertEveryRequestWasHandled,
  recordUnhandledRequest,
} from "src/test/msw/unhandled_requests";

beforeAll(async () => {
  await worker.start({ quiet: true, onUnhandledFrame: recordUnhandledRequest });
});

afterEach(async () => {
  await cleanup();
  worker.resetHandlers();
  assertEveryRequestWasHandled();
});

afterAll(async () => {
  assertEveryRequestWasHandled();
  await worker.stop();
});
