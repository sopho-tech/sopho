import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebooks/Notebook/Cell";
import { SophoTable, ColumnConfig } from "src/components/SophoTable/SophoTable";
import CellOutputStles from "src/components/Notebooks/Notebook/CellOutput/CellOutput.module.css";

interface CellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: CellOutputProps) {
  const { getOutput, getOutputState } = useCellOutputStore();
  const output = getOutput(cellId);
  const outputState = getOutputState(cellId);

  const columns: ColumnConfig<Record<string, any>>[] =
    output?.column_names?.map((column_name) => ({
      key: column_name,
      header: column_name,
      type: "accessor" as const,
      accessor: column_name,
      cell: (props: any) => props.getValue(),
    })) ?? [];

  const data = output?.data ?? [];

  if (outputState === CellOutputState.ABSENT || output === undefined)
    return null;

  return (
    <div className={CellOutputStles.container}>
      <SophoTable
        columns={columns}
        data={data}
        tableHeaderCellStyle={CellOutputStles.tableHeaderCell}
        tableDataCellStyle={CellOutputStles.tableDataCell}
      />
    </div>
  );
}
