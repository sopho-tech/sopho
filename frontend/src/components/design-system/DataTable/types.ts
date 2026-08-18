import { PaginationState, OnChangeFn, CellContext } from "@tanstack/react-table";
import { ColumnDataType } from "src/constants/database_types";

export enum TableType {
  FULL = "FULL",
  SERVER_SIDE_PAGINATED = "SERVER_SIDE_PAGINATED",
  CLIENT_SIDE_PAGINATED = "CLIENT_SIDE_PAGINATED",
}

export type ColumnConfig<T> = {
  key: string;
  header: string;
  size?: number;
  minSize?: number;
  maxSize?: number;
  fixedWidth?: number;
  type: "accessor" | "display";
  cell?: (props: CellContext<T, unknown>) => React.ReactNode;
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
  showRowsPerPage?: boolean;
};

export type DataTableProps<T> = {
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
  showRowsPerPage?: boolean;
  enableColumnResizing?: boolean;
  enableRowDragging?: boolean;
  showRowNumbers?: boolean;
  getRowId?: (row: T) => string;
  emptyMessage?: string;
  emptySearchMessage?: string;
};

