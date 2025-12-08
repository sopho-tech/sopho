import { useState, useEffect } from "react";
import { useParams } from "react-router";
import RGL, { WidthProvider, Layout } from "react-grid-layout";
import { getInstanceByDom } from "echarts";
import { useCellsByNotebookId } from "src/api/notebook/queries";
import {
  useDashboardByCanvasId,
  useUpdateDashboard,
} from "src/api/dashboard/queries";
import { useCanvasStore } from "src/components/Canvases/store";
import { CellType } from "src/components/Notebook/Cell";
import { DashboardChart } from "src/components/Dashboard/DashboardChart";
import {
  convertRGLayoutToDto,
  convertDtoToRGLayout,
} from "src/components/Dashboard/dto";
import { Button, Flex, Icon } from "src/components/design-system";
import styles from "src/components/Dashboard/Dashboard.module.css";
import "react-grid-layout/css/styles.css";

const ReactGridLayout = WidthProvider(RGL);

export function Dashboard() {
  const params = useParams();
  const canvasId = params.id || "";
  const { activeNotebookId } = useCanvasStore();
  const cellsByNotebookIdQuery = useCellsByNotebookId(
    activeNotebookId,
    CellType.CHART
  );
  const dashboardQuery = useDashboardByCanvasId(canvasId);
  const updateDashboardMutation = useUpdateDashboard();
  const cells = cellsByNotebookIdQuery.data?.cells ?? [];
  const [layout, setLayout] = useState<Layout[]>([]);

  useEffect(() => {
    if (dashboardQuery.data?.layout) {
      setLayout(convertDtoToRGLayout(dashboardQuery.data.layout));
    }
  }, [dashboardQuery.data]);

  const handleLayoutChange = (newLayout: Layout[]) => {
    setLayout(newLayout);
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
    <Flex direction="column" gap="md">
      <Button
        label="Save"
        shape="rectangle"
        size="sm"
        backgroundColor="accent"
        onClick={() => {
          if (!dashboardQuery.data || !activeNotebookId) {
            return;
          }
          const layoutDto = convertRGLayoutToDto(layout, activeNotebookId);
          updateDashboardMutation.mutate({
            dashboardId: dashboardQuery.data.id,
            payload: {
              ...dashboardQuery.data,
              layout: layoutDto,
            },
          });
        }}
      />
      <ReactGridLayout
        className={`${styles.layout} ${styles.layoutEditing}`}
        layout={layout}
        cols={12}
        rowHeight={100}
        margin={[10, 10]}
        onLayoutChange={handleLayoutChange}
        onDragStart={handleDragStart}
        onDrag={handleDrag}
        onDragStop={handleDragStop}
        onResize={handleDrag}
        onResizeStop={handleDrag}
        useCSSTransforms={true}
        draggableHandle={`.${styles.dragHandle}`}
      >
        {cells.map((cell) => (
          <div key={cell.id} className={styles.gridItem}>
            <div className={styles.dragHandle}>
              <Icon type="grip_vertical" color="grey" size="sm" />
            </div>
            <DashboardChart cellId={cell.id} />
          </div>
        ))}
      </ReactGridLayout>
    </Flex>
  );
}
