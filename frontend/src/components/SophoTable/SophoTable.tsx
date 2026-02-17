import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
  PaginationState,
  OnChangeFn,
  CellContext,
} from "@tanstack/react-table";
import styles from "src/components/SophoTable/SophoTable.module.css";
import { Pagination } from "src/components/design-system/Pagination";

export enum TableType {
  FULL = "FULL",
  PAGINATED = "PAGINATED",
}

export type ColumnConfig<T> = {
  key: string;
  header: string;
  size?: number;
  type: "accessor" | "display";
  cell?: (props: CellContext<T, unknown>) => React.ReactNode;
  accessor?: keyof T;
};

export type PaginationConfig = {
  pagination: PaginationState;
  currentPage: number;
  totalItems: number;
  totalPages: number;
  pageSize: number;
  onPaginationChange: OnChangeFn<PaginationState>;
  onChangePageSize: (newValue: string) => void;
  onPageClick: (newPage: number) => void;
  paginationContainerClassName?: string;
};

type SophoTableProps<T> = {
  tableType?: TableType;
  columns: ColumnConfig<T>[];
  data: T[];
  isLoading?: boolean;
  isError?: boolean;
  loadingComponent?: React.ReactNode;
  errorComponent?: React.ReactNode;
  overallContainerStyle?: string;
  tableContainerStyle?: string;
  tableHeaderCellStyle?: string;
  tableDataCellStyle?: string;
  tableFirstHeaderCellStyle?: string;
  tableLastHeaderCellStyle?: string;
  paginationConfig?: PaginationConfig;
  onRowClick?: (row: T) => void;
};

function useCreateReactTable<T>(
  tableType: TableType,
  reactTableColumns: ColumnDef<T, unknown>[],
  data: T[],
  paginationConfig: PaginationConfig | undefined
) {
  const paginationState = (() => {
    if (tableType === TableType.PAGINATED) {
      if (!paginationConfig) {
        throw Error("Pagination config is required for paginated table");
      }
      return {
        rowCount: paginationConfig.totalItems,
        pageCount: paginationConfig.totalPages,
        state: { pagination: paginationConfig.pagination },
        onPaginationChange: paginationConfig.onPaginationChange,
        manualPagination: true as const,
      };
    }
    return {};
  })();

  return useReactTable({
    columns: reactTableColumns,
    data: data ?? [],
    getCoreRowModel: getCoreRowModel(),
    ...paginationState,
  });
}

export function SophoTable<T>({
  tableType = TableType.FULL,
  columns,
  data,
  isLoading = false,
  isError = false,
  loadingComponent = <div>Loading...</div>,
  errorComponent = <div>Error fetching data</div>,
  overallContainerStyle,
  tableHeaderCellStyle,
  tableDataCellStyle,
  tableFirstHeaderCellStyle,
  tableLastHeaderCellStyle,
  paginationConfig,
  tableContainerStyle,
  onRowClick,
}: SophoTableProps<T>) {
  const columnHelper = createColumnHelper<T>();

  const reactTableColumns: ColumnDef<T, unknown>[] = columns.map((col) => {
    if (col.type === "accessor") {
        return columnHelper.accessor(
          col.accessor as Parameters<typeof columnHelper.accessor>[0],
          {
            header: col.header,
            cell: col.cell || ((props: CellContext<T, unknown>) => props.getValue()),
            size: col.size,
          }
        );
    } else {
      return columnHelper.display({
        id: col.key,
        header: col.header,
        cell: col.cell,
        size: col.size,
      });
    }
  });

  const table = useCreateReactTable(
    tableType,
    reactTableColumns,
    data,
    paginationConfig
  );

  if (isLoading) {
    return <>{loadingComponent}</>;
  }

  if (isError) {
    return <>{errorComponent}</>;
  }

  return (
    <div
      className={`${styles.overallContainer} ${overallContainerStyle || ""}`}
    >
      <div className={`${styles.tableContainer} ${tableContainerStyle}`}>
        <table className={styles.table}>
          <thead className={styles.thead}>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header, index) => {
                  const isFirst = index === 0;
                  const isLast = index === headerGroup.headers.length - 1;
                  return (
                    <th
                      key={header.id}
                      className={`${styles.tableHeaderCell} ${tableHeaderCellStyle || ""} ${isFirst && tableFirstHeaderCellStyle ? tableFirstHeaderCellStyle : ""} ${isLast && tableLastHeaderCellStyle ? tableLastHeaderCellStyle : ""}`}
                      style={{ width: header.getSize() }}
                    >
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody className={styles.tableBody}>
            {table.getRowModel().rows.map((row) => (
              <tr
                key={row.id}
                className={styles.tableBodyRow}
                onClick={() => onRowClick?.(row.original)}
                style={{ cursor: onRowClick ? "pointer" : "default" }}
              >
                {row.getVisibleCells().map((cell) => (
                  <td
                    key={cell.id}
                    className={`${styles.tableDataCell} ${tableDataCellStyle}`}
                    style={{ width: cell.column.getSize() }}
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {paginationConfig && (
        <Pagination
          totalPages={paginationConfig.totalPages}
          currentPage={paginationConfig.currentPage}
          pageSize={paginationConfig.pageSize}
          totalItems={paginationConfig.totalItems}
          onChangePageSize={paginationConfig.onChangePageSize}
          onPageClick={paginationConfig.onPageClick}
          containerClassName={paginationConfig.paginationContainerClassName}
        />
      )}
    </div>
  );
}
