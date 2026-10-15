import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  useTable,
  PaginationState,
  OnChangeFn,
  CellContext,
  RowData,
  columnSizingFeature,
  rowPaginationFeature,
  tableFeatures,
} from "@tanstack/react-table";
import styles from "src/components/Table/Table.module.css";
import { Pagination } from "src/components/design-system/Pagination";

const tableComponentFeatures = tableFeatures({
  columnSizingFeature,
  rowPaginationFeature,
});

type TableComponentFeatures = typeof tableComponentFeatures;

type TableComponentColumnDef<T extends RowData> = ColumnDef<
  TableComponentFeatures,
  T,
  unknown
>;

export enum TableType {
  FULL = "FULL",
  PAGINATED = "PAGINATED",
}

export type TableSize = "default" | "compact";

export type TableVariant = "bordered" | "plain";

export const rowHoverClasses = {
  reveal: styles.rowHoverReveal,
  hide: styles.rowHoverHide,
};

export type ColumnConfig<T extends RowData> = {
  key: string;
  header: string;
  size?: number;
  type: "accessor" | "display";
  cell?: (
    props: CellContext<TableComponentFeatures, T, unknown>,
  ) => React.ReactNode;
  accessor?: keyof T;
  fill?: boolean;
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

type TableProps<T extends RowData> = {
  tableType?: TableType;
  size?: TableSize;
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
  variant?: TableVariant;
  showHeader?: boolean;
  isRowSelected?: (row: T) => boolean;
};

function useCreateReactTable<T extends RowData>(
  tableType: TableType,
  reactTableColumns: TableComponentColumnDef<T>[],
  data: T[],
  paginationConfig: PaginationConfig | undefined,
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

  return useTable({
    features: tableComponentFeatures,
    columns: reactTableColumns,
    data: data ?? [],
    ...paginationState,
  });
}

export function Table<T extends RowData>({
  tableType = TableType.FULL,
  size = "default",
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
  variant = "bordered",
  showHeader = true,
  isRowSelected,
}: TableProps<T>) {
  const isCompact = size === "compact";
  const isPlain = variant === "plain";
  const columnHelper = createColumnHelper<TableComponentFeatures, T>();

  const columnConfigMap = new Map<string, ColumnConfig<T>>();
  columns.forEach((col) => {
    columnConfigMap.set(col.key, col);
    if (col.type === "accessor" && col.accessor) {
      columnConfigMap.set(String(col.accessor), col);
    }
  });

  const getColumnWidth = (columnId: string, fallback: number) =>
    columnConfigMap.get(columnId)?.fill ? "100%" : fallback;

  const reactTableColumns: TableComponentColumnDef<T>[] = columns.map((col) => {
    if (col.type === "accessor") {
      return columnHelper.accessor(
        col.accessor as Parameters<typeof columnHelper.accessor>[0],
        {
          header: col.header,
          cell:
            col.cell ||
            ((props: CellContext<TableComponentFeatures, T, unknown>) =>
              props.getValue()),
          size: col.size,
        },
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
    paginationConfig,
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
      <div
        className={`${styles.tableContainer} ${tableContainerStyle} ${isPlain ? styles.tableContainerPlain : ""}`}
      >
        <table
          className={`${styles.table} ${isPlain ? styles.tablePlain : ""}`}
        >
          {showHeader && (
            <thead className={styles.thead}>
              {table.getHeaderGroups().map((headerGroup) => (
                <tr key={headerGroup.id}>
                  {headerGroup.headers.map((header, index) => {
                    const isFirst = index === 0;
                    const isLast = index === headerGroup.headers.length - 1;
                    return (
                      <th
                        key={header.id}
                        className={`${styles.tableHeaderCell} ${isCompact ? styles.tableHeaderCellCompact : ""} ${tableHeaderCellStyle || ""} ${isFirst && tableFirstHeaderCellStyle ? tableFirstHeaderCellStyle : ""} ${isLast && tableLastHeaderCellStyle ? tableLastHeaderCellStyle : ""}`}
                        style={{
                          width: getColumnWidth(
                            header.column.id,
                            header.getSize(),
                          ),
                        }}
                      >
                        {header.isPlaceholder
                          ? null
                          : flexRender(
                              header.column.columnDef.header,
                              header.getContext(),
                            )}
                      </th>
                    );
                  })}
                </tr>
              ))}
            </thead>
          )}
          <tbody className={styles.tableBody}>
            {table.getRowModel().rows.map((row) => (
              <tr
                key={row.id}
                className={`${styles.tableBodyRow} ${isRowSelected?.(row.original) ? styles.tableBodyRowSelected : ""}`}
                onClick={() => onRowClick?.(row.original)}
                style={{ cursor: onRowClick ? "pointer" : "default" }}
              >
                {row.getAllCells().map((cell) => (
                  <td
                    key={cell.id}
                    className={`${styles.tableDataCell} ${isCompact ? styles.tableDataCellCompact : ""} ${isPlain ? styles.tableDataCellPlain : ""} ${tableDataCellStyle}`}
                    style={{
                      width: getColumnWidth(
                        cell.column.id,
                        cell.column.getSize(),
                      ),
                    }}
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
