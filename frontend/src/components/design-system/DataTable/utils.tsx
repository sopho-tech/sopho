import {
  ColumnDef,
  getCoreRowModel,
  useReactTable,
  PaginationState,
  OnChangeFn,
  getSortedRowModel,
  SortingState,
  getFilteredRowModel,
  getPaginationRowModel,
  Table,
  Row,
} from "@tanstack/react-table";
import { ColumnDataType } from "src/constants/database_types";
import { Pagination } from "src/components/design-system/Pagination";
import {
  TableType,
  PaginationConfig,
} from "src/components/design-system/DataTable/types";
import styles from "src/components/design-system/DataTable/DataTable.module.css";

export function getAlignmentClass(
  dataType: ColumnDataType | undefined,
  cellValue: unknown
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

export function getHeaderAlignmentClass(
  dataType: ColumnDataType | undefined
): string {
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

export function getHeaderJustifyContent(
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

export function getFontClass(dataType: ColumnDataType | undefined): string {
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

export function getDisplayRowNumber<T>(table: Table<T>, row: Row<T>): number {
  const { pageIndex, pageSize } = table.getState().pagination;
  const positionInCurrentPage = table
    .getRowModel()
    .rows.findIndex((rowInCurrentPage) => rowInCurrentPage.id === row.id);

  if (positionInCurrentPage === -1) {
    throw Error(`Row ${row.id} is not part of the rendered rows`);
  }

  return pageIndex * pageSize + positionInCurrentPage + 1;
}

export function useCreateReactTable<T>(
  tableType: TableType,
  reactTableColumns: ColumnDef<T, unknown>[],
  data: T[],
  paginationConfig: PaginationConfig | undefined,
  setSorting: OnChangeFn<SortingState>,
  sorting: SortingState,
  globalFilter: string,
  setGlobalFilter: OnChangeFn<string>,
  pagination: PaginationState | undefined,
  setPagination: OnChangeFn<PaginationState> | undefined,
  enableColumnResizing: boolean = true,
  getRowId?: (row: T) => string
) {
  const baseConfig = {
    columns: reactTableColumns,
    data: data ?? [],
    getCoreRowModel: getCoreRowModel(),
    ...(getRowId && { getRowId }),
  };

  const fullConfig =
    tableType === TableType.FULL
      ? {
          ...baseConfig,
          getSortedRowModel: getSortedRowModel(),
          getFilteredRowModel: getFilteredRowModel(),
          onSortingChange: setSorting,
          globalFilterFn: "auto" as const,
          state: { globalFilter, sorting },
          onGlobalFilterChange: setGlobalFilter,
          isMultiSortEvent: () => true,
          columnResizeMode: "onChange" as const,
          columnResizeDirection: "ltr" as const,
          enableColumnResizing,
        }
      : tableType === TableType.SERVER_SIDE_PAGINATED
        ? (() => {
            if (!paginationConfig) {
              throw Error("Pagination config is required for paginated table");
            }
            return {
              ...baseConfig,
              rowCount: paginationConfig.totalItems,
              pageCount: paginationConfig.totalPages,
              state: { pagination: paginationConfig.pagination },
              onPaginationChange: paginationConfig.onPaginationChange,
              manualPagination: true,
            };
          })()
        : (() => {
            if (!pagination || !setPagination) {
              throw Error(
                "Pagination state is required for client-side paginated table"
              );
            }
            return {
              ...baseConfig,
              getSortedRowModel: getSortedRowModel(),
              getFilteredRowModel: getFilteredRowModel(),
              getPaginationRowModel: getPaginationRowModel(),
              onSortingChange: setSorting,
              onPaginationChange: setPagination,
              globalFilterFn: "auto" as const,
              state: { pagination, globalFilter, sorting },
              onGlobalFilterChange: setGlobalFilter,
              isMultiSortEvent: () => true,
              columnResizeMode: "onChange" as const,
              columnResizeDirection: "ltr" as const,
              enableColumnResizing,
              defaultColumn: { minSize: 200 },
            };
          })();

  return useReactTable(fullConfig);
}

export function renderPaginationControl<T>(
  tableType: TableType,
  paginationConfig: PaginationConfig | undefined,
  table: Table<T>,
  showRowsPerPage: boolean = true
): React.ReactNode {
  const onChangePageSize = (newPageSize: string) => {
    table.setPageSize(Number(newPageSize));
  };
  const onPageClick = (newPageIndex: number) => {
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
        showRowsPerPage={showRowsPerPage}
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
        showRowsPerPage={paginationConfig.showRowsPerPage ?? true}
      />
    );
  }
  return null;
}
