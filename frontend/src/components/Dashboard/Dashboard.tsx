import { useEffect } from "react";
import { useParams } from "react-router";
import RGL, { WidthProvider, Layout } from "react-grid-layout";
import { getInstanceByDom } from "echarts";
import { useCellsByNotebookId } from "src/api/notebook/queries";
import { useDashboardByCanvasId } from "src/api/dashboard/queries";
import { useCanvasStore } from "src/components/Canvases/store";
import { CellType } from "src/components/Notebook/Cell";
import { DashboardChart } from "src/components/Dashboard/DashboardChart";
import { convertDtoToRGLayout } from "src/components/Dashboard/dto";
import {
  useDashboardStore,
  DashboardMode,
} from "src/components/Dashboard/store";
import { useDashboardSave } from "src/components/Dashboard/hooks";
import { Icon } from "src/components/design-system";
import styles from "src/components/Dashboard/Dashboard.module.css";
import "react-grid-layout/css/styles.css";

const ReactGridLayout = WidthProvider(RGL);

export function Dashboard() {
  const params = useParams();
  const canvasId = params.id || "";
  const { activeNotebookId } = useCanvasStore();
  const { mode, layout, setLayout } = useDashboardStore();
  const cellsByNotebookIdQuery = useCellsByNotebookId(
    activeNotebookId,
    CellType.CHART
  );
  const dashboardQuery = useDashboardByCanvasId(canvasId);
  const cells = cellsByNotebookIdQuery.data?.cells ?? [];
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
    gridItems.forEach((item) => {
      const divs = item.querySelectorAll("div");
      divs.forEach((div) => {
        const chart = getInstanceByDom(div as HTMLElement);
        if (chart) {
          chart.resize();
        }
      });
    });
  };

  const handleDragStart = () => {
    document.body.style.userSelect = "none";
    document.body.style.webkitUserSelect = "none";
  };

  const handleDragStop = () => {
    document.body.style.userSelect = "";
    document.body.style.webkitUserSelect = "";
    handleDrag();
  };

  if (cellsByNotebookIdQuery.isLoading || layout.length === 0) {
    return null;
  }

  return (
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
    >
      {cells.map((cell) => (
        <div key={cell.id} className={styles.gridItem}>
          {isEditing && (
            <div className={styles.dragHandle}>
              <Icon type="grip_vertical" color="grey" size="sm" />
            </div>
          )}
          <DashboardChart cellId={cell.id} />
        </div>
      ))}
    </ReactGridLayout>
  );
}
