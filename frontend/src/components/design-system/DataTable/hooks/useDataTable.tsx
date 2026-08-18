import { useState, useMemo } from "react";
import {
  createColumnHelper,
  ColumnDef,
  Table,
  PaginationState,
  SortingState,
  OnChangeFn,
} from "@tanstack/react-table";
import {
  TableType,
  ColumnConfig,
  PaginationConfig,
} from "src/components/design-system/DataTable/types";
import {
  useCreateReactTable,
  getDisplayRowNumber,
} from "src/components/design-system/DataTable/utils";
import { Icon } from "src/components/design-system/Icon/Icon";
import { Flex } from "src/components/design-system/Flex";
import { Text } from "src/components/design-system/Text";
import { getIconForDataType } from "src/utils/column_utils";

const ROW_NUMBER_COLUMN_KEY = "row_number";
const ROW_NUMBER_COLUMN_WIDTH = 64;

type UseDataTableReturn<T> = {
  table: Table<T>;
  columnConfigMap: Map<string, ColumnConfig<T>>;
  globalFilter: string;
  setGlobalFilter: (value: string) => void;
  sorting: SortingState;
  setSorting: (
    value: SortingState | ((prev: SortingState) => SortingState)
  ) => void;
  pagination: PaginationState | undefined;
  setPagination: (value: PaginationState | undefined) => void;
};

export function useDataTable<T>({
  tableType,
  columns,
  data,
  paginationConfig,
  enableColumnResizing,
  enableRowDragging,
  showRowNumbers,
  getRowId,
}: {
  tableType: TableType;
  columns: ColumnConfig<T>[];
  data: T[];
  paginationConfig?: PaginationConfig;
  enableColumnResizing?: boolean;
  enableRowDragging?: boolean;
  showRowNumbers?: boolean;
  getRowId?: (row: T) => string;
}): UseDataTableReturn<T> {
  const [sorting, setSorting] = useState<SortingState>([]);
  const [globalFilter, setGlobalFilter] = useState<string>("");
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

  const columnsWithLeadingUtilityColumns = useMemo(() => {
    const leadingUtilityColumns: ColumnConfig<T>[] = [];

    if (enableRowDragging) {
      leadingUtilityColumns.push({
        key: "drag",
        header: "",
        type: "display",
        cell: () => <Icon type="grip_vertical" color="grey" size="sm" />,
        size: 10,
      });
    }

    if (showRowNumbers) {
      leadingUtilityColumns.push({
        key: ROW_NUMBER_COLUMN_KEY,
        header: "",
        type: "display",
        cell: ({ table, row }) => (
          <Flex justifyContent="flex-end">
            <Text color="darkGrey">{getDisplayRowNumber(table, row)}</Text>
          </Flex>
        ),
        fixedWidth: ROW_NUMBER_COLUMN_WIDTH,
      });
    }

    return [...leadingUtilityColumns, ...columns];
  }, [columns, enableRowDragging, showRowNumbers]);

  const columnConfigMap = useMemo(() => {
    const map = new Map<string, ColumnConfig<T>>();
    columnsWithLeadingUtilityColumns.forEach((col) => {
      map.set(col.key, col);
      if (col.type === "accessor" && col.accessor) {
        map.set(String(col.accessor), col);
      }
    });
    return map;
  }, [columnsWithLeadingUtilityColumns]);

  const reactTableColumns: ColumnDef<T, unknown>[] = useMemo(() => {
    return columnsWithLeadingUtilityColumns.map((col) => {
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

      const columnDef: Partial<ColumnDef<T, unknown>> = {
        size: col.size,
        minSize: col.minSize,
        maxSize: col.maxSize,
        enableResizing: col.fixedWidth === undefined,
      };

      if (col.type === "accessor" && col.accessor) {
        const accessorKey = String(col.accessor);
        return columnHelper.accessor(
          (row: T) => (row as Record<string, unknown>)[accessorKey] as unknown,
          {
            id: col.key,
            ...columnDef,
            header: headerContent,
            cell:
              col.cell ||
              ((props: { getValue: () => unknown }) => props.getValue()),
          }
        );
      } else {
        return columnHelper.display({
          id: col.key,
          ...columnDef,
          header: headerContent,
          cell: col.cell as ColumnDef<T, unknown>["cell"],
        });
      }
    });
  }, [columnsWithLeadingUtilityColumns, columnHelper]);

  const onPaginationChange: OnChangeFn<PaginationState> = (updaterOrValue) => {
    setPagination(
      typeof updaterOrValue === "function"
        ? updaterOrValue(pagination ?? { pageIndex: 0, pageSize: 10 })
        : updaterOrValue
    );
  };

  const table = useCreateReactTable(
    tableType,
    reactTableColumns,
    data,
    paginationConfig,
    setSorting,
    sorting,
    globalFilter,
    setGlobalFilter,
    pagination,
    onPaginationChange,
    enableColumnResizing ?? true,
    getRowId
  );

  return {
    table,
    columnConfigMap,
    globalFilter,
    setGlobalFilter,
    sorting,
    setSorting,
    pagination,
    setPagination,
  } as UseDataTableReturn<T>;
}
