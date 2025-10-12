import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebooks/Notebook/Cell";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import tableStyles from "src/css/table.module.css";
import CellOutputStles from "src/components/Notebooks/Notebook/CellOutput/CellOutput.module.css";

interface CellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: CellOutputProps) {
  const { getOutput, getOutputState } = useCellOutputStore();
  const output = getOutput(cellId);
  const outputState = getOutputState(cellId);
  const columnHelper = createColumnHelper<Record<string, any>>();
  const columns = output?.column_names?.map((column_name) => {
    return columnHelper.accessor((x) => x[column_name], {
      id: column_name,
      cell: (props: any) => props.getValue(),
    });
  });
  const data = output?.data;
  const table = useReactTable({
    columns: columns ?? [],
    data: data ?? [],
    getCoreRowModel: getCoreRowModel(),
  });

  if (outputState === CellOutputState.ABSENT || output === undefined)
    return null;

  return (
    <div className={CellOutputStles.container}>
      <table className={tableStyles.table}>
        <thead>
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <th key={header.id} className={tableStyles["table th"]}>
                  {header.isPlaceholder
                    ? null
                    : flexRender(
                        header.column.columnDef.header,
                        header.getContext()
                      )}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => (
            <tr key={row.id} className={tableStyles["table tbody tr"]}>
              {row.getVisibleCells().map((cell) => (
                <td key={cell.id} className={tableStyles["table td"]}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
