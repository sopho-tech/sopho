import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
  PaginationState,
  OnChangeFn,
  getSortedRowModel,
  SortingState,
  getFilteredRowModel,
  getPaginationRowModel,
  Table,
} from "@tanstack/react-table";
import { useState } from "react";
import styles from "src/components/design-system/DataTable/DataTable.module.css";
import { Pagination } from "src/components/Pagination";
import { ColumnDataType } from "src/constants/database_types";
import { Icon } from "src/components/design-system/Icon/Icon";
import { Flex } from "src/components/design-system/Flex";
import { Input } from "src/components/design-system/Input";
import { Text } from "src/components/design-system/Text";
import { getIconForDataType } from "src/utils/column_utils";  

export enum TableType {
  FULL = "FULL",
  SERVER_SIDE_PAGINATED = "SERVER_SIDE_PAGINATED",
  CLIENT_SIDE_PAGINATED = "CLIENT_SIDE_PAGINATED",
}

export type ColumnConfig<T> = {
  key: string;
  header: string;
  size?: number;
  type: "accessor" | "display";
  cell?: (props: any) => React.ReactNode;
  accessor?: keyof T;
  dataType?: ColumnDataType;
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

type DataTableProps<T> = {
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
};

function getAlignmentClass(
  dataType: ColumnDataType | undefined,
  cellValue: any
): string {
  if (!dataType) {
    return styles.cellAlignLeft;
  }

  if (dataType === ColumnDataType.BOOL) {
    return cellValue === true ? styles.cellAlignLeft : styles.cellAlignRight;
  }

  if (
    dataType === ColumnDataType.INT4 ||
    dataType === ColumnDataType.INT8 ||
    dataType === ColumnDataType.TIMESTAMP ||
    dataType === ColumnDataType.TIMESTAMPTZ
  ) {
    return styles.cellAlignRight;
  }

  return styles.cellAlignLeft;
}

function getHeaderAlignmentClass(dataType: ColumnDataType | undefined): string {
  if (!dataType) {
    return styles.cellAlignLeft;
  }

  if (
    dataType === ColumnDataType.INT4 ||
    dataType === ColumnDataType.INT8 ||
    dataType === ColumnDataType.TIMESTAMP ||
    dataType === ColumnDataType.TIMESTAMPTZ
  ) {
    return styles.cellAlignRight;
  }

  return styles.cellAlignLeft;
}

function getHeaderJustifyContent(
  dataType: ColumnDataType | undefined
): "flex-start" | "flex-end" {
  if (!dataType) {
    return "flex-start";
  }

  if (
    dataType === ColumnDataType.INT4 ||
    dataType === ColumnDataType.INT8 ||
    dataType === ColumnDataType.TIMESTAMP ||
    dataType === ColumnDataType.TIMESTAMPTZ
  ) {
    return "flex-end";
  }

  return "flex-start";
}

function getFontClass(dataType: ColumnDataType | undefined): string {
  if (!dataType) {
    return "";
  }

  if (
    dataType === ColumnDataType.INT4 ||
    dataType === ColumnDataType.INT8 ||
    dataType === ColumnDataType.TIMESTAMP ||
    dataType === ColumnDataType.TIMESTAMPTZ
  ) {
    return styles.cellFontMono;
  }

  return "";
}

function createReactTable<T>(
  tableType: TableType,
  reactTableColumns: ColumnDef<T, any>[],
  data: T[],
  paginationConfig: PaginationConfig | undefined,
  setSorting: OnChangeFn<SortingState>,
  sorting: SortingState,
  globalFilter: any,
  setGlobalFilter: any,
  pagination: PaginationState | undefined,
  setPagination: any | undefined
) {
  switch (tableType) {
    case TableType.FULL: {
      return useReactTable({
        columns: reactTableColumns,
        data: data ?? [],
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        getFilteredRowModel: getFilteredRowModel(),
        onSortingChange: setSorting,
        globalFilterFn: "auto",
        state: {
          globalFilter,
          sorting,
        },
        onGlobalFilterChange: setGlobalFilter,
        isMultiSortEvent: () => true,
        columnResizeMode: "onChange",
        columnResizeDirection: "ltr",
        enableColumnResizing: true,
        defaultColumn: {
          minSize: 200,
        },
      });
    }
    case TableType.SERVER_SIDE_PAGINATED: {
      if (!paginationConfig) {
        throw Error("Pagination config is required for paginated table");
      }
      return useReactTable({
        columns: reactTableColumns,
        data: data ?? [],
        getCoreRowModel: getCoreRowModel(),
        rowCount: paginationConfig.totalItems,
        pageCount: paginationConfig.totalPages,
        state: {
          pagination: paginationConfig.pagination,
        },
        onPaginationChange: paginationConfig.onPaginationChange,
        manualPagination: true,
      });
    }
    case TableType.CLIENT_SIDE_PAGINATED: {
      if (!pagination) {
        throw Error(
          "Pagination state is required for client-side paginated table"
        );
      }
      return useReactTable({
        columns: reactTableColumns,
        data: data ?? [],
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        getFilteredRowModel: getFilteredRowModel(),
        getPaginationRowModel: getPaginationRowModel(),
        onSortingChange: setSorting,
        onPaginationChange: setPagination,
        globalFilterFn: "auto",
        state: {
          pagination: pagination,
          globalFilter,
          sorting,
        },
        onGlobalFilterChange: setGlobalFilter,
        isMultiSortEvent: () => true,
        columnResizeMode: "onChange",
        columnResizeDirection: "ltr",
        enableColumnResizing: true,
        defaultColumn: {
          minSize: 200,
        },
      });
    }
  }
}

function renderPaginationControl<T>(
  tableType: TableType,
  paginationConfig: PaginationConfig | undefined,
  table: Table<T>
) {
  const onChangePageSize = (newPageSize: string) => {
    table.setPageSize(Number(newPageSize));
  };
  const onPageClick = (newPageIndex: number) => {
    console.log(newPageIndex);
    table.setPageIndex(newPageIndex);
  };
  if (tableType == TableType.CLIENT_SIDE_PAGINATED) {
    return (
      <Pagination
        totalPages={table.getPageCount()}
        currentPage={table.getState().pagination.pageIndex}
        pageSize={table.getState().pagination.pageSize}
        totalItems={table.getRowCount()}
        onChangePageSize={onChangePageSize}
        onPageClick={onPageClick}
        containerClassName={styles.paginationControl}
      />
    );
  }
  if (tableType == TableType.SERVER_SIDE_PAGINATED && paginationConfig) {
    return (
      <Pagination
        totalPages={paginationConfig.totalPages}
        currentPage={paginationConfig.currentPage}
        pageSize={paginationConfig.pageSize}
        totalItems={paginationConfig.totalItems}
        onChangePageSize={paginationConfig.onChangePageSize}
        onPageClick={paginationConfig.onPageClick}
        containerClassName={paginationConfig.paginationContainerClassName}
      />
    );
  }
}

export function DataTable<T>({
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
}: DataTableProps<T>) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const [globalFilter, setGlobalFilter] = useState<any>([]);
  const [pagination, setPagination] = useState<PaginationState | undefined>(
    () => {
      if (tableType === TableType.CLIENT_SIDE_PAGINATED) {
        return {
          pageIndex: 0,
          pageSize: 10,
        };
      }
      return undefined;
    }
  );
  const columnHelper = createColumnHelper<T>();

  const columnConfigMap = new Map<string, ColumnConfig<T>>();
  columns.forEach((col) => {
    columnConfigMap.set(col.key, col);
    if (col.type === "accessor" && col.accessor) {
      columnConfigMap.set(String(col.accessor), col);
    }
  });

  const reactTableColumns: ColumnDef<T, any>[] = columns.map((col) => {
    const headerContent = col.dataType
      ? () => {
          const dataType = col.dataType!;
          return (
            <Flex direction="row" gap="2xs" alignItems="center">
              <Icon
                type={getIconForDataType(dataType)}
                color="default"
                strokeWidth={2}
                size="sm"
              />
              <Text>{col.header}</Text>
            </Flex>
          );
        }
      : col.header;

    if (col.type === "accessor") {
      return columnHelper.accessor(col.accessor as any, {
        header: headerContent,
        cell: col.cell || ((props) => props.getValue()),
      });
    } else {
      return columnHelper.display({
        id: col.key,
        header: headerContent,
        cell: col.cell,
      });
    }
  });

  const table = createReactTable(
    tableType,
    reactTableColumns,
    data,
    paginationConfig,
    setSorting,
    sorting,
    globalFilter,
    setGlobalFilter,
    pagination,
    setPagination
  );

  const [hoveredColumnId, setHoveredColumnId] = useState<string | null>(null);
  const isResizingColumn = Boolean(
    table.getState().columnSizingInfo?.isResizingColumn
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
      <Flex gap="lg" justifyContent="space-between">
        <Input
          value={globalFilter}
          onChange={(e) => table.setGlobalFilter(String(e.target.value))}
          placeholder="Search..."
          leadingIcon="search"
        />
        {renderPaginationControl(tableType, paginationConfig, table)}
      </Flex>
      <div
        className={`${styles.tableContainer} ${tableContainerStyle} ${
          isResizingColumn ? styles.isResizingColumn : ""
        }`}
      >
        <table
          className={styles.table}
          style={{
            width: table.getCenterTotalSize(),
          }}
        >
          <thead className={styles.thead}>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header, index) => {
                  const isFirst = index === 0;
                  const isLast = index === headerGroup.headers.length - 1;
                  const isHovered = hoveredColumnId === header.column.id;
                  const columnConfig = columnConfigMap.get(header.column.id);
                  const dataType = columnConfig?.dataType;
                  const alignmentClass = getHeaderAlignmentClass(dataType);
                  const justifyContent = getHeaderJustifyContent(dataType);
                  return (
                    <th
                      key={header.id}
                      colSpan={header.colSpan}
                      className={`${styles.tableHeaderCell} ${tableHeaderCellStyle || ""} ${isFirst && tableFirstHeaderCellStyle ? tableFirstHeaderCellStyle : ""} ${isLast && tableLastHeaderCellStyle ? tableLastHeaderCellStyle : ""} ${isHovered ? styles.tableHeaderCellHovered : ""} ${alignmentClass}`}
                      style={{
                        width: header.getSize(),
                      }}
                    >
                      {header.isPlaceholder ? null : (
                        <Flex
                          alignItems="center"
                          justifyContent={justifyContent}
                          gap="2xs"
                          sx={
                            header.column.getToggleSortingHandler()
                              ? { cursor: "pointer", userSelect: "none" }
                              : undefined
                          }
                          onClick={header.column.getToggleSortingHandler()}
                          onMouseEnter={() => {
                            if (!isResizingColumn) {
                              setHoveredColumnId(header.column.id);
                            }
                          }}
                          onMouseLeave={() => {
                            if (!isResizingColumn) {
                              setHoveredColumnId(null);
                            }
                          }}
                          onMouseDown={(e) => {
                            if (header.column.getToggleSortingHandler()) {
                              e.preventDefault();
                            }
                          }}
                        >
                          {flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                          {(() => {
                            const sortState = header.column.getIsSorted();
                            const isSortable =
                              !!header.column.getToggleSortingHandler();
                            const showHoverHint =
                              !sortState &&
                              isHovered &&
                              isSortable &&
                              !isResizingColumn;

                            const shouldShowIcon = sortState || showHoverHint;

                            if (!shouldShowIcon) {
                              return null;
                            }

                            const iconType =
                              sortState === "asc"
                                ? "arrow_up"
                                : sortState === "desc"
                                  ? "arrow_down"
                                  : "swap_vert";

                            return (
                              <span className={`${styles.sortIconSlot}`}>
                                <Icon
                                  type={iconType}
                                  color="black"
                                  size="sm"
                                  strokeWidth={3}
                                />
                              </span>
                            );
                          })()}
                        </Flex>
                      )}
                      {table.options.enableColumnResizing && (
                        <div
                          onDoubleClick={() => header.column.resetSize()}
                          onMouseDown={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            header.getResizeHandler()(e);
                          }}
                          onTouchStart={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            header.getResizeHandler()(e);
                          }}
                          className={`${styles.resizer} ${
                            table.options.columnResizeDirection === "ltr"
                              ? styles.ltr
                              : styles.rtl
                          } ${
                            header.column.getIsResizing()
                              ? styles.isResizing
                              : ""
                          }`}
                        ></div>
                      )}
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody className={styles.tableBody}>
            {table.getRowModel().rows.map((row) => (
              <tr key={row.id} className={styles.tableBodyRow}>
                {row.getVisibleCells().map((cell) => {
                  const isHovered = hoveredColumnId === cell.column.id;
                  const columnConfig = columnConfigMap.get(cell.column.id);
                  const dataType = columnConfig?.dataType;
                  const cellValue = cell.getValue();
                  const alignmentClass = getAlignmentClass(dataType, cellValue);
                  const fontClass = getFontClass(dataType);
                  return (
                    <td
                      key={cell.id}
                      className={`${styles.tableDataCell} ${tableDataCellStyle} ${isHovered ? styles.tableDataCellHovered : ""} ${alignmentClass} ${fontClass}`}
                      style={{
                        width: cell.column.getSize(),
                      }}
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
