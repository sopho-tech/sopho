import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebooks/Notebook/Cell";
import { SophoTable, ColumnConfig } from "src/components/SophoTable/SophoTable";
import CellOutputStles from "src/components/Notebooks/Notebook/CellOutput/CellOutput.module.css";
import CellStyles from "src/css/cell.module.css";

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
    })) ?? [];

  const data = output?.data ?? [];

  if (
    outputState === CellOutputState.ABSENT ||
    outputState === undefined ||
    output === undefined
  )
    return null;

  return (
    <SophoTable
      columns={columns}
      data={data}
      overallContainerStyle={CellStyles.outputContainer}
      tableContainerStyle={CellOutputStles.container}
      tableHeaderCellStyle={CellOutputStles.tableHeaderCell}
      tableDataCellStyle={CellOutputStles.tableDataCell}
    />
  );
}
