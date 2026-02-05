import { useEffect } from "react";
import { useParams } from "react-router";
import RGL, { WidthProvider, Layout } from "react-grid-layout";
import { useDashboardByCanvasId } from "src/api/dashboard/queries";
import { DashboardChart } from "src/components/Dashboard/DashboardChart";
import { DashboardToolbar } from "src/components/Dashboard/DashboardToolbar";
import { ChartBrowser } from "src/components/Dashboard/ChartBrowser";
import {
  convertDtoToRGLayout,
  DEFAULT_CHART_WIDTH,
  DEFAULT_CHART_HEIGHT,
} from "src/components/Dashboard/dto";
import {
  useDashboardStore,
  DashboardMode,
} from "src/components/Dashboard/store";
import { useDashboardSave } from "src/components/Dashboard/hooks";
import styles from "src/components/Dashboard/Dashboard.module.css";
import "react-grid-layout/css/styles.css";
import { Flex } from "src/components/design-system";

const ReactGridLayout = WidthProvider(RGL);

export function Dashboard() {
  const params = useParams();
  const canvasId = params.id || "";
  const { mode, setLayout, getLayout } = useDashboardStore();
  const layout = getLayout();
  const dashboardQuery = useDashboardByCanvasId(canvasId);
  const isEditing = mode === DashboardMode.EDITING;

  useDashboardSave(dashboardQuery.data);

  useEffect(() => {
    if (dashboardQuery.data?.layout) {
      setLayout(convertDtoToRGLayout(dashboardQuery.data.layout));
    }
  }, [dashboardQuery.data, setLayout]);

  const handleLayoutChange = (newLayout: Layout[]) => {
    if (isEditing) {
      setLayout(newLayout);
    }
  };

  const handleDrag = () => {
    const gridItems = document.querySelectorAll(".react-grid-item");
    // gridItems.forEach((item) => {
    //   const divs = item.querySelectorAll("div");
    // });
  };

  const handleDragStart = () => {
    document.body.style.userSelect = "none";
  };

  const handleDragStop = () => {
    document.body.style.userSelect = "";
    handleDrag();
  };

  const handleOnDropDragOver = () => {
    return { w: DEFAULT_CHART_WIDTH, h: DEFAULT_CHART_HEIGHT };
  };

  const handleOnDrop = (
    droppedLayout: Layout[],
    itemLayout: Layout,
    e: DragEvent
  ) => {
    if (!e.dataTransfer) {
      return;
    }
    const cellId = e.dataTransfer.getData("text/plain");
    const latestLayout = getLayout();
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
  };

  return (
    <Flex gap="2xs" flex="grow">
      <ReactGridLayout
        className={`${styles.layout} ${isEditing ? styles.layoutEditing : ""}`}
        layout={layout}
        cols={12}
        rowHeight={100}
        margin={[10, 10]}
        containerPadding={[10, 10]}
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
            <DashboardChart cellId={layoutItem.i} />
          </div>
        ))}
      </ReactGridLayout>
      <ChartBrowser />
      <DashboardToolbar />
    </Flex>
  );
}
