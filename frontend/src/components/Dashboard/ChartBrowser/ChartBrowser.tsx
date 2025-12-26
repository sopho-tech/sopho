import { Flex } from "src/components/design-system";
import { DashboardMode, useDashboardStore } from "../store";
import { useCellsByNotebookId } from "src/api/notebook/queries";
import {
  getChartContent,
  CellDto,
  CellType,
} from "src/components/Notebook/Cell/dto";
import {
  DataTable,
  TableType,
  ColumnConfig,
} from "src/components/design-system/DataTable";
import { useCanvasStore } from "src/components/Canvases/store";
import { useMemo } from "react";

type ChartCellRow = {
  id: string;
  name: string | null;
  chartType: string | undefined;
};

export function ChartBrowser() {
  const { showChartBrowser, mode, getLayout } = useDashboardStore();
  const { activeNotebookId } = useCanvasStore();
  const cellsQuery = useCellsByNotebookId(activeNotebookId, CellType.CHART);
  const layout = getLayout();

  const tableData = useMemo<ChartCellRow[]>(() => {
    const cells = cellsQuery.data?.cells ?? [];
    const cellsInDashboard = new Set(layout.map((item) => item.i));
    return cells
      .filter((cell: CellDto) => !cellsInDashboard.has(cell.id))
      .map((cell: CellDto) => {
        const chartContent = getChartContent(cell);
        return {
          id: cell.id,
          name: cell.name,
          chartType: chartContent?.chart_type,
        };
      });
  }, [layout, cellsQuery.data]);

  const columns: ColumnConfig<ChartCellRow>[] = useMemo(
    () => [
      {
        key: "name",
        header: "Cell Name",
        type: "accessor",
        accessor: "name",
        cell: (props) => props.getValue() || "Unnamed",
      },
      {
        key: "chartType",
        header: "Chart Type",
        type: "accessor",
        accessor: "chartType",
        cell: (props) => props.getValue() || "N/A",
      },
    ],
    []
  );

  if (!showChartBrowser || mode == DashboardMode.VIEWING) {
    return null;
  }

  return (
    <Flex
      border="default"
      borderRadius="lg"
      paddingX="sm"
      paddingY="sm"
      shadow="2xs"
      width="20%"
    >
      <DataTable
        tableType={TableType.CLIENT_SIDE_PAGINATED}
        columns={columns}
        data={tableData}
        isLoading={cellsQuery.isLoading}
        isError={cellsQuery.isError}
        showRowsPerPage={false}
        enableColumnResizing={false}
        enableRowDragging={true}
        getRowId={(row) => row.id}
      />
    </Flex>
  );
}
