import { setupServer } from "msw/node";
import { handlers } from "src/test/msw/handlers";

export const server = setupServer(...handlers);
