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

export function createReactTable<T>(
  tableType: TableType,
  reactTableColumns: ColumnDef<T, any>[],
  data: T[],
  paginationConfig: PaginationConfig | undefined,
  setSorting: OnChangeFn<SortingState>,
  sorting: SortingState,
  globalFilter: any,
  setGlobalFilter: any,
  pagination: PaginationState | undefined,
  setPagination: any | undefined,
  enableColumnResizing: boolean = true,
  getRowId?: (row: T) => string
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
        enableColumnResizing: enableColumnResizing,
        // defaultColumn: {
        //   minSize: 200,
        // },
        ...(getRowId && { getRowId }),
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
        ...(getRowId && { getRowId }),
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
        enableColumnResizing: enableColumnResizing,
        defaultColumn: {
          minSize: 200,
        },
        ...(getRowId && { getRowId }),
      });
    }
  }
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
