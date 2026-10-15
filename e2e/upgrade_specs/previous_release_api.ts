import { expect, request, type APIRequestContext } from "@playwright/test";
import {
  ADMIN_EMAIL,
  ADMIN_PASSWORD,
  DEMO_CONNECTION_NAME,
  ORDER_COUNT_SQL,
} from "e2e/support/constants";

export const PREVIOUS_RELEASE_ROUTES = {
  SESSION: "/api/v1/auth/session",
  CONNECTIONS: "/api/v1/connection",
  CANVASES: "/api/v1/canvas",
  NOTEBOOKS_BY_CANVAS: "/api/v1/notebook/canvas/:canvasId",
  CELLS: "/api/v1/cell",
} as const;

export type SeededCanvas = {
  canvasId: string;
  canvasName: string;
  sqlCellId: string;
};

type IdentifiedRecord = { id: string; name: string };

async function readJson<T>(
  response: Awaited<ReturnType<APIRequestContext["get"]>>,
  expectedStatus: number,
): Promise<T> {
  expect(response.status(), await response.text()).toBe(expectedStatus);
  return (await response.json()) as T;
}

export async function seedCanvasThroughPreviousRelease(
  baseUrl: string,
  canvasName: string,
): Promise<SeededCanvas> {
  const previousRelease = await request.newContext({ baseURL: baseUrl });
  const signedIn = await previousRelease.post(PREVIOUS_RELEASE_ROUTES.SESSION, {
    data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
  });
  expect(signedIn.status(), await signedIn.text()).toBe(200);
  const connections = await readJson<IdentifiedRecord[]>(
    await previousRelease.get(PREVIOUS_RELEASE_ROUTES.CONNECTIONS),
    200,
  );
  const demoConnection = connections.find(
    (connection) => connection.name === DEMO_CONNECTION_NAME,
  );
  if (!demoConnection) {
    throw new Error(
      `the previous release did not seed ${DEMO_CONNECTION_NAME}`,
    );
  }
  const canvas = await readJson<IdentifiedRecord>(
    await previousRelease.post(PREVIOUS_RELEASE_ROUTES.CANVASES, {
      data: { name: canvasName, description: "Seeded before the upgrade" },
    }),
    201,
  );
  const notebooks = await readJson<IdentifiedRecord[]>(
    await previousRelease.get(
      PREVIOUS_RELEASE_ROUTES.NOTEBOOKS_BY_CANVAS.replace(
        ":canvasId",
        canvas.id,
      ),
    ),
    200,
  );
  const sqlCell = await readJson<IdentifiedRecord>(
    await previousRelease.post(PREVIOUS_RELEASE_ROUTES.CELLS, {
      data: {
        notebook_id: notebooks[0].id,
        connection_id: demoConnection.id,
        name: "Order count",
        content: ORDER_COUNT_SQL,
        cell_type: "SQL",
      },
    }),
    201,
  );
  await previousRelease.dispose();
  return { canvasId: canvas.id, canvasName, sqlCellId: sqlCell.id };
}
