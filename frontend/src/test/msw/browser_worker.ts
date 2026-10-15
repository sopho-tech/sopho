import { setupWorker } from "msw/browser";
import { handlers } from "src/test/msw/handlers";

export const worker = setupWorker(...handlers);
