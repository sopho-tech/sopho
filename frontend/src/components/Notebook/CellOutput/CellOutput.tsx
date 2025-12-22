import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebook/Cell";
import { DataTable, ColumnConfig, Box } from "src/components/design-system";
import CellOutputStles from "src/components/Notebook/CellOutput/CellOutput.module.css";
import CellStyles from "src/css/cell.module.css";
import { TableType } from "src/components/design-system/DataTable";

interface CellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: CellOutputProps) {
  const { getOutput, getOutputState } = useCellOutputStore();
  const output = getOutput(cellId);
  const outputState = getOutputState(cellId);

  const columns: ColumnConfig<Record<string, any>>[] =
    output?.columns?.map((column) => ({
      key: column.column_name,
      header: column.column_name,
      type: "accessor" as const,
      accessor: column.column_name,
      cell: (props: any) => props.getValue(),
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
