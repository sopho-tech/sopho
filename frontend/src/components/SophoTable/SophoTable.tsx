import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import styles from "src/components/SophoTable/SophoTable.module.css";

export type ColumnConfig<T> = {
  key: string;
  header: string;
  size?: number;
  type: "accessor" | "display";
  cell?: (props: any) => React.ReactNode;
  accessor?: keyof T;
};

type SophoTableProps<T> = {
  columns: ColumnConfig<T>[];
  data: T[];
  isLoading?: boolean;
  isError?: boolean;
  loadingComponent?: React.ReactNode;
  errorComponent?: React.ReactNode;
};

export function SophoTable<T>({
  columns,
  data,
  isLoading = false,
  isError = false,
  loadingComponent = <div>Loading...</div>,
  errorComponent = <div>Error fetching data</div>,
}: SophoTableProps<T>) {
  const columnHelper = createColumnHelper<T>();

  const reactTableColumns: ColumnDef<T, any>[] = columns.map((col) => {
    if (col.type === "accessor") {
      return columnHelper.accessor(col.accessor as any, {
        header: col.header,
        cell: col.cell || ((props) => props.getValue()),
        size: col.size,
      });
    } else {
      return columnHelper.display({
        id: col.key,
        header: col.header,
        cell: col.cell,
        size: col.size,
      });
    }
  });

  const table = useReactTable({
    columns: reactTableColumns,
    data: data ?? [],
    getCoreRowModel: getCoreRowModel(),
  });

  if (isLoading) {
    return <>{loadingComponent}</>;
  }

  if (isError) {
    return <>{errorComponent}</>;
  }

  return (
    <table className={styles.table}>
      <thead>
        {table.getHeaderGroups().map((headerGroup) => (
          <tr key={headerGroup.id}>
            {headerGroup.headers.map((header) => (
              <th key={header.id} style={{ width: header.getSize() }}>
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
          <tr key={row.id}>
            {row.getVisibleCells().map((cell) => (
              <td key={cell.id} style={{ width: cell.column.getSize() }}>
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
