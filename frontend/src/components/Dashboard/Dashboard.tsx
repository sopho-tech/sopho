import { useCallback, useEffect, useMemo } from "react";
import { useNavigate, useParams } from "react-router";
import RGL, { WidthProvider, Layout } from "react-grid-layout";
import { useDashboardByCanvasId } from "src/api/dashboard/queries";
import { useCellsByNotebookId } from "src/api/notebook/queries";
import { CellType } from "src/components/Notebook/Cell/dto";
import { DashboardChart } from "src/components/Dashboard/DashboardChart";
import { DashboardToolbar } from "src/components/Dashboard/DashboardToolbar";
import { ChartBrowser } from "src/components/Dashboard/ChartBrowser";
import {
  convertDtoToRGLayout,
  GRID_COLUMN_COUNT,
  DEFAULT_CHART_WIDTH,
  DEFAULT_CHART_HEIGHT,
} from "src/components/Dashboard/dto";
import { useDashboardSave, useDashboardReset } from "src/components/Dashboard/hooks";
import styles from "src/components/Dashboard/Dashboard.module.css";
import "react-grid-layout/css/styles.css";
import { EmptyState } from "src/components/EmptyState";
import { Flex } from "src/components/design-system";
import { useStore, DashboardMode } from "src/store";
import { APP_ROUTES } from "src/constants/app_routes";

const ReactGridLayout = WidthProvider(RGL);

export function Dashboard() {
  const params = useParams();
  const navigate = useNavigate();
  const canvasId = params.id || "";
  const mode = useStore((state) => state.dashboard.mode);
  const setLayout = useStore((state) => state.dashboard.setLayout);
  const layout = useStore((state) => state.dashboard.layout);
  const activeNotebookId = useStore((state) => state.canvas.activeNotebookId);
  const dashboardQuery = useDashboardByCanvasId(canvasId);
  const cellsQuery = useCellsByNotebookId(activeNotebookId, CellType.CHART);
  const isEditing = mode === DashboardMode.EDITING;
  const { handleEditSaveClick } = useDashboardReset(canvasId, true);
  const setMode = useStore((state) => state.dashboard.setMode);
  const setShowChartBrowser = useStore(
    (state) => state.dashboard.setShowChartBrowser
  );

  useDashboardSave(dashboardQuery.data);

  useEffect(
    () => () => {
      setMode(DashboardMode.VIEWING);
      setShowChartBrowser(false);
    },
    [setMode, setShowChartBrowser]
  );

  const handleNavigateToNotebook = useCallback(
    () =>
      navigate(
        `${APP_ROUTES.CANVAS.replace(":id", canvasId)}/${APP_ROUTES.CANVAS_ROUTES.NOTEBOOK}`
      ),
    [navigate, canvasId]
  );

  const chartCellCount = cellsQuery.data?.cells?.length ?? 0;
  const hasNoChartCells = chartCellCount === 0;
  const hasEmptyLayout = layout.length === 0;
  const showNoChartCellsEmptyState = hasNoChartCells;
  const showNoChartsInLayoutEmptyState =
    !hasNoChartCells && hasEmptyLayout && !isEditing;

  useEffect(() => {
    if (dashboardQuery.data?.layout) {
      setLayout(convertDtoToRGLayout(dashboardQuery.data.layout));
    } else {
      setLayout([]);
    }
  }, [dashboardQuery.data, setLayout]);

  const handleLayoutChange = useCallback(
    (newLayout: Layout[]) => {
      if (isEditing) {
        setLayout(newLayout);
      }
    },
    [isEditing, setLayout]
  );

  const handleDrag = useCallback(() => {}, []);

  const handleDragStart = useCallback(() => {
    document.body.style.userSelect = "none";
  }, []);

  const handleDragStop = useCallback(() => {
    document.body.style.userSelect = "";
    handleDrag();
  }, [handleDrag]);

  const handleOnDropDragOver = useCallback(
    () => ({ w: DEFAULT_CHART_WIDTH, h: DEFAULT_CHART_HEIGHT }),
    []
  );

  const handleOnDrop = useCallback(
    (droppedLayout: Layout[], itemLayout: Layout, e: DragEvent) => {
      if (!e.dataTransfer) {
        return;
      }
      const cellId = e.dataTransfer.getData("application/x-sopho-cell-id");
      const latestLayout = useStore.getState().dashboard.layout;
      const existingIds = new Set(latestLayout.map((item) => item.i));
      const newLayout = droppedLayout.map((item) => {
        const isNewItem = !existingIds.has(cellId);
        const matchesPosition =
          item.x === itemLayout.x &&
          item.y === itemLayout.y &&
          item.w === itemLayout.w &&
          item.h === itemLayout.h;
        if (isNewItem && matchesPosition) {
          return { ...item, i: cellId };
        }
        return item;
      });
      setLayout(newLayout);
    },
    [setLayout]
  );

  const margin = useMemo<[number, number]>(() => [20, 20], []);
  const containerPadding = useMemo<[number, number]>(() => [0, 0], []);

  const renderContent = () => {
    if (showNoChartCellsEmptyState) {
      return (
        <EmptyState
          icon="layout_dashboard"
          heading="No chart cells yet"
          description="Create chart cells in the notebook to add them to your dashboard"
          buttonLabel="Go to Notebook"
          onButtonClick={handleNavigateToNotebook}
        />
      );
    }
    if (showNoChartsInLayoutEmptyState) {
      return (
        <EmptyState
          icon="layout_dashboard"
          heading="No charts on dashboard"
          description="Add chart cells from your notebook to the dashboard"
          buttonLabel="Add charts"
          onButtonClick={handleEditSaveClick}
        />
      );
    }
    return (
      <ReactGridLayout
        className={`${styles.layout} ${isEditing ? styles.layoutEditing : ""}`}
        layout={layout}
        cols={GRID_COLUMN_COUNT}
        rowHeight={100}
        margin={margin}
        containerPadding={containerPadding}
        onLayoutChange={handleLayoutChange}
        onDragStart={handleDragStart}
        onDrag={handleDrag}
        onDragStop={handleDragStop}
        onResize={handleDrag}
        onResizeStop={handleDrag}
        useCSSTransforms={true}
        draggableHandle={`.${styles.dragHandle}`}
        isDraggable={isEditing}
        isResizable={isEditing}
        onDrop={handleOnDrop}
        onDropDragOver={handleOnDropDragOver}
        isDroppable={true}
      >
        {layout.map((layoutItem) => (
          <div key={layoutItem.i} className={styles.gridItem}>
            <DashboardChart
              cellId={layoutItem.i}
              dashboardId={dashboardQuery.data?.id ?? ""}
            />
          </div>
        ))}
      </ReactGridLayout>
    );
  };

  return (
    <Flex gap="2xs" flex="grow">
      <Flex flex="grow" alignItems="stretch">
        {renderContent()}
      </Flex>
      <ChartBrowser />
      <DashboardToolbar />
    </Flex>
  );
}
