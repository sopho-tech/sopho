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
  TEST_USER,
  notebookWithCells,
} from "src/test/test_data";
import { testTenantApiUrl } from "src/test/tenant_routes";

export const signedInSessionHandler = http.get(
  API_ENDPOINTS.AUTH.GET_SESSION,
  () => new HttpResponse(null, { status: 200 }),
);

export const signedOutSessionHandler = http.get(
  API_ENDPOINTS.AUTH.GET_SESSION,
  () => HttpResponse.json({ message: "Unauthorized" }, { status: 401 }),
);

export const currentUserHandler = http.get(
  API_ENDPOINTS.AUTH.GET_PROFILE_DETAILS,
  () => HttpResponse.json(TEST_USER),
);

export const emaillessSignInPageOptionsHandler = http.get(
  API_ENDPOINTS.AUTH.SIGN_IN_PAGE_OPTIONS,
  () =>
    HttpResponse.json({
      providers: [],
      password_sign_in_available: true,
      password_reset_available: false,
    }),
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
    http.get(testTenantApiUrl(API_ENDPOINTS.CANVAS.GET_BY_ID), () =>
      HttpResponse.json(CANVAS),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.NOTEBOOK.GET_BY_CANVAS_ID), () =>
      HttpResponse.json([notebookWithCells(cells)]),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.NOTEBOOK.GET_BY_ID), () =>
      HttpResponse.json(notebookWithCells(cells)),
    ),
    http.get(
      testTenantApiUrl(API_ENDPOINTS.NOTEBOOK.GET_CELLS_BY_NOTEBOOK_ID),
      ({ request }) => {
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
      },
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.DASHBOARD.GET_BY_CANVAS_ID), () =>
      HttpResponse.json(dashboard),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.CELL.GET_BY_ID), ({ params }) =>
      HttpResponse.json(
        requireByIdentifier(cellsByIdentifier, params.id, "cell"),
      ),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.CONNECTION.GET_ALL), () =>
      HttpResponse.json([SQLITE_CONNECTION]),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.AI_CONFIGURATION.GET), () =>
      HttpResponse.json({ status: "not_configured" }),
    ),
    http.get(testTenantApiUrl(API_ENDPOINTS.AI_SUMMARY.GET_BY_DASHBOARD), () =>
      HttpResponse.json({ dashboard: null, charts: [] }),
    ),
    http.post(testTenantApiUrl(API_ENDPOINTS.CELL.EXECUTE), ({ params }) =>
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

export const handlers = [
  signedInSessionHandler,
  currentUserHandler,
  emaillessSignInPageOptionsHandler,
];
