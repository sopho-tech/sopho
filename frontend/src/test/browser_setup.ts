import "src/css/index.css";
import type * as Shaders from "@paper-design/shaders-react";
import { createElement } from "react";
import { afterAll, afterEach, beforeAll, vi } from "vitest";
import { cleanup } from "vitest-browser-react";
import { worker } from "src/test/msw/browser_worker";
import {
  assertEveryRequestWasHandled,
  recordUnhandledRequest,
} from "src/test/msw/unhandled_requests";

vi.mock("@paper-design/shaders-react", async (importOriginal) => {
  const shaders = await importOriginal<typeof Shaders>();
  const MeshGradientWithoutAnimation = (props: Shaders.MeshGradientProps) =>
    createElement(shaders.MeshGradient, { ...props, speed: 0 });
  return { ...shaders, MeshGradient: MeshGradientWithoutAnimation };
});

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
