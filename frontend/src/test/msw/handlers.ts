import { http, HttpResponse } from "msw";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import type { DashboardDto } from "src/components/Dashboard/dto";
import type {
  CellDto,
  CellType,
  ExecuteCellResponseDto,
} from "src/components/Notebook/Cell/dto";
import {
  CANVAS,
  SQLITE_CONNECTION,
  notebookWithCells,
} from "src/test/test_data";

export const signedInSessionHandler = http.get(
  API_ENDPOINTS.AUTH.GET_SESSION,
  () => new HttpResponse(null, { status: 200 }),
);

export const signedOutSessionHandler = http.get(
  API_ENDPOINTS.AUTH.GET_SESSION,
  () => HttpResponse.json({ message: "Unauthorized" }, { status: 401 }),
);

type CanvasPageHandlersOptions = {
  cells: CellDto[];
  dashboard: DashboardDto;
  executionResults: Record<string, ExecuteCellResponseDto>;
};

function requireByIdentifier<T>(
  itemsByIdentifier: Record<string, T>,
  identifier: unknown,
  kind: string,
): T {
  const item = itemsByIdentifier[String(identifier)];
  if (item === undefined) {
    throw new Error(`the test did not define ${kind} ${String(identifier)}`);
  }
  return item;
}

export function canvasPageHandlers({
  cells,
  dashboard,
  executionResults,
}: CanvasPageHandlersOptions) {
  const cellsByIdentifier = Object.fromEntries(
    cells.map((cell) => [cell.id, cell]),
  );
  return [
    http.get(API_ENDPOINTS.CANVAS.GET_BY_ID, () => HttpResponse.json(CANVAS)),
    http.get(API_ENDPOINTS.NOTEBOOK.GET_BY_CANVAS_ID, () =>
      HttpResponse.json([notebookWithCells(cells)]),
    ),
    http.get(API_ENDPOINTS.NOTEBOOK.GET_BY_ID, () =>
      HttpResponse.json(notebookWithCells(cells)),
    ),
    http.get(API_ENDPOINTS.NOTEBOOK.GET_CELLS_BY_NOTEBOOK_ID, ({ request }) => {
      const requestedCellType = new URL(request.url).searchParams.get(
        "cell_type",
      ) as CellType | null;
      return HttpResponse.json(
        notebookWithCells(
          cells.filter(
            (cell) =>
              requestedCellType === null ||
              cell.cell_type === requestedCellType,
          ),
        ),
      );
    }),
    http.get(API_ENDPOINTS.DASHBOARD.GET_BY_CANVAS_ID, () =>
      HttpResponse.json(dashboard),
    ),
    http.get(API_ENDPOINTS.CELL.GET_BY_ID, ({ params }) =>
      HttpResponse.json(
        requireByIdentifier(cellsByIdentifier, params.id, "cell"),
      ),
    ),
    http.get(API_ENDPOINTS.CONNECTION.GET_ALL, () =>
      HttpResponse.json([SQLITE_CONNECTION]),
    ),
    http.get(API_ENDPOINTS.AI_CONFIGURATION.GET, () =>
      HttpResponse.json({ status: "not_configured" }),
    ),
    http.get(API_ENDPOINTS.AI_SUMMARY.GET_BY_DASHBOARD, () =>
      HttpResponse.json({ dashboard: null, charts: [] }),
    ),
    http.post(API_ENDPOINTS.CELL.EXECUTE, ({ params }) =>
      HttpResponse.json(
        requireByIdentifier(
          executionResults,
          params.id,
          "an execution result for cell",
        ),
      ),
    ),
  ];
}

export const handlers = [signedInSessionHandler];
