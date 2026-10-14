import { expect, type APIRequestContext } from "@playwright/test";
import {
  ADMIN_EMAIL,
  ADMIN_PASSWORD,
  API_ROUTES,
  DEMO_CONNECTION_NAME,
  FAKE_LLM_API_KEY,
} from "e2e/support/constants";

type IdentifiedRecord = { id: string; name: string };

export type CreatedCanvas = {
  canvasId: string;
  notebookId: string;
  dashboard: Record<string, unknown> & { id: string };
};

async function readJson<T>(
  response: Awaited<ReturnType<APIRequestContext["get"]>>,
  expectedStatus: number,
): Promise<T> {
  expect(response.status(), await response.text()).toBe(expectedStatus);
  return (await response.json()) as T;
}

export async function signIn(request: APIRequestContext): Promise<void> {
  const response = await request.post(API_ROUTES.SESSION, {
    data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
  });
  expect(response.status(), await response.text()).toBe(200);
}

export async function findDemoConnection(
  request: APIRequestContext,
): Promise<IdentifiedRecord> {
  const connections = await readJson<IdentifiedRecord[]>(
    await request.get(API_ROUTES.CONNECTIONS),
    200,
  );
  const demoConnection = connections.find(
    (connection) => connection.name === DEMO_CONNECTION_NAME,
  );
  if (!demoConnection) {
    throw new Error(`the ${DEMO_CONNECTION_NAME} connection is not seeded`);
  }
  return demoConnection;
}

export async function createCanvas(
  request: APIRequestContext,
  name: string,
): Promise<CreatedCanvas> {
  const canvas = await readJson<IdentifiedRecord>(
    await request.post(API_ROUTES.CANVASES, {
      data: { name, description: "Created by the end-to-end tests" },
    }),
    201,
  );
  const notebooks = await readJson<IdentifiedRecord[]>(
    await request.get(
      API_ROUTES.NOTEBOOKS_BY_CANVAS.replace(":canvasId", canvas.id),
    ),
    200,
  );
  const dashboard = await readJson<CreatedCanvas["dashboard"]>(
    await request.get(
      API_ROUTES.DASHBOARD_BY_CANVAS.replace(":canvasId", canvas.id),
    ),
    200,
  );
  return { canvasId: canvas.id, notebookId: notebooks[0].id, dashboard };
}

export async function createSqlCell(
  request: APIRequestContext,
  notebookId: string,
  connectionId: string,
  name: string,
  query: string,
): Promise<IdentifiedRecord> {
  return readJson<IdentifiedRecord>(
    await request.post(API_ROUTES.CELLS, {
      data: {
        notebook_id: notebookId,
        connection_id: connectionId,
        name,
        content: query,
        cell_type: "SQL",
      },
    }),
    201,
  );
}

export async function createOrderStatusChart(
  request: APIRequestContext,
  notebookId: string,
  connectionId: string,
  sqlCellId: string,
): Promise<IdentifiedRecord> {
  return readJson<IdentifiedRecord>(
    await request.post(API_ROUTES.CELLS, {
      data: {
        notebook_id: notebookId,
        connection_id: connectionId,
        name: "Orders by status chart",
        cell_type: "CHART",
        content: JSON.stringify({
          chart_type: "BAR",
          cell_id: sqlCellId,
          x_axis: "order_status",
          x_axis_alias: "order_status",
          series: [
            {
              column: "order_count",
              aggregate_function: "SUM",
              label: "Orders",
              alias: "order_count_sum",
              color_index: null,
            },
          ],
          x_axis_title: null,
          y_axis_title: null,
          y_axis_sort_by: "order_count_sum",
          y_axis_sort_order: "DESC",
        }),
      },
    }),
    201,
  );
}

export async function placeChartOnDashboard(
  request: APIRequestContext,
  canvas: CreatedCanvas,
  chartCellId: string,
): Promise<void> {
  const response = await request.put(
    API_ROUTES.DASHBOARD.replace(":dashboardId", canvas.dashboard.id),
    {
      data: {
        ...canvas.dashboard,
        layout: [
          {
            cell_id: chartCellId,
            notebook_id: canvas.notebookId,
            x_position: 0,
            y_position: 0,
            x_size: 6,
            y_size: 4,
          },
        ],
      },
    },
  );
  expect(response.status(), await response.text()).toBe(200);
}

export async function configureAnthropicProvider(
  request: APIRequestContext,
): Promise<void> {
  const configuration = await readJson<{ status: string }>(
    await request.put(API_ROUTES.AI_CONFIGURATION, {
      data: { provider: "anthropic", api_key: FAKE_LLM_API_KEY },
    }),
    200,
  );
  expect(configuration.status).toBe("live");
}
