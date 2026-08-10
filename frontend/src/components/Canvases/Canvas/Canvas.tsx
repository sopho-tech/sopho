import { useCallback, useEffect } from "react";
import { Outlet, useLocation, useNavigate, useParams } from "react-router";
import { useCanvas, useUpdateCanvas } from "src/api/canvas/queries";
import { useDashboardByCanvasId } from "src/api/dashboard/queries";
import { useNotebooksByCanvasId } from "src/api/notebook/queries";
import { Flex, InlineEdit } from "src/components/design-system";
import { useStore } from "src/store";
import { AiSummaryRow } from "src/components/Dashboard/AiSummaryRow";
import { CanvasButtons } from "src/components/Canvases/CanvasButtons";
import { APP_ROUTES } from "src/constants/app_routes";

export function Canvas() {
  const params = useParams();
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const query = useCanvas(params.id!);
  const dashboardQuery = useDashboardByCanvasId(params.id!);
  const notebooksQuery = useNotebooksByCanvasId(params.id!);
  const updateCanvas = useUpdateCanvas();
  const setActiveNotebookId = useStore(
    (state) => state.canvas.setActiveNotebookId
  );

  const isDashboardView = pathname.endsWith(
    `/${APP_ROUTES.CANVAS_ROUTES.DASHBOARD}`
  );

  useEffect(() => {
    const firstNotebookId =
      notebooksQuery.data &&
      notebooksQuery.data.length > 0 &&
      notebooksQuery.data[0].id
        ? notebooksQuery.data[0].id
        : "";
    setActiveNotebookId(firstNotebookId);
  }, [notebooksQuery.data, setActiveNotebookId]);

  const handleNameSave = useCallback(
    (name: string) =>
      updateCanvas.mutate({
        canvasId: params.id!,
        payload: { ...query.data!, name },
      }),
    [params.id, query.data, updateCanvas]
  );

  const handleDescriptionSave = useCallback(
    (description: string) =>
      updateCanvas.mutate({
        canvasId: params.id!,
        payload: { ...query.data!, description },
      }),
    [params.id, query.data, updateCanvas]
  );

  const handleViewTypeChange = useCallback(
    (view: string) =>
      navigate(`${APP_ROUTES.CANVAS.replace(":id", params.id!)}/${view}`),
    [navigate, params.id]
  );

  if (!query.data) {
    return <span>No data available</span>;
  }

  return (
    <Flex
      direction="column"
      paddingX="2xl"
      paddingY="md"
      gap="xl"
      flex="grow"
      overflow="scrollY"
    >
      <Flex direction="column" gap="sm">
        <Flex direction="row" justifyContent="space-between">
          <InlineEdit
            value={query.data.name ?? ""}
            onSave={handleNameSave}
            headingLevel={1}
            clearButtonSize="md"
            defaultValue="Default Canvas Title"
          />
          <CanvasButtons
            viewType={
              isDashboardView
                ? APP_ROUTES.CANVAS_ROUTES.DASHBOARD
                : APP_ROUTES.CANVAS_ROUTES.NOTEBOOK
            }
            onViewTypeChange={handleViewTypeChange}
          />
        </Flex>
        <InlineEdit
          value={query.data.description ?? ""}
          onSave={handleDescriptionSave}
          placeholder="Add a description"
          defaultValue="Default description"
          textColor="subtle"
        />
        {isDashboardView &&
          dashboardQuery.data?.id &&
          (dashboardQuery.data.layout?.length ?? 0) > 0 && (
            <AiSummaryRow dashboardId={dashboardQuery.data.id} />
          )}
      </Flex>
      <Outlet />
    </Flex>
  );
}
