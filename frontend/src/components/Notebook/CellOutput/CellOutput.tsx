import { CellOutputState } from "src/components/Notebook/Cell";
import { useStore } from "src/store";
import { DataTable, ColumnConfig, Box } from "src/components/design-system";
import CellOutputStles from "src/components/Notebook/CellOutput/CellOutput.module.css";
import CellStyles from "src/css/cell.module.css";
import { TableType } from "src/components/design-system/DataTable";

interface CellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: CellOutputProps) {
  const output = useStore((state) => state.cell.outputs[cellId]);
  const outputState = useStore((state) => state.cell.outputStates[cellId]);

  const columns: ColumnConfig<Record<string, unknown>>[] =
    output?.columns?.map((column) => ({
      key: column.column_name,
      header: column.column_name,
      type: "accessor" as const,
      accessor: column.column_name,
      cell: (props) => props.getValue() as React.ReactNode,
      dataType: column.data_type,
    })) ?? [];

  const data = output?.data ?? [];

  if (
    outputState === CellOutputState.ABSENT ||
    outputState === undefined ||
    output === undefined
  )
    return null;

  return (
    <Box paddingX="lg" paddingY="lg">
      <DataTable
        tableType={TableType.CLIENT_SIDE_PAGINATED}
        columns={columns}
        data={data}
        overallContainerStyle={`${CellStyles.outputContainerSql} ${CellOutputStles.outputContainer}`}
        tableContainerStyle={`${CellOutputStles.tableContainer}`}
        tableHeaderCellStyle={CellOutputStles.tableHeaderCell}
        tableDataCellStyle={CellOutputStles.tableDataCell}
      />
    </Box>
  );
}
