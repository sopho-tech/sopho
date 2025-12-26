import { useState } from "react";
import {
  TableType,
  DataTableProps,
} from "src/components/design-system/DataTable/types";
import { renderPaginationControl } from "src/components/design-system/DataTable/utils";
import { useDataTable } from "src/components/design-system/DataTable/hooks/useDataTable";
import { useRowDragging } from "src/components/design-system/DataTable/hooks/useRowDragging";
import { TableHeader } from "src/components/design-system/DataTable/components/TableHeader";
import { TableBody } from "src/components/design-system/DataTable/components/TableBody";
import { Input } from "src/components/design-system/Input";
import { Flex } from "src/components/design-system/Flex";
import styles from "src/components/design-system/DataTable/DataTable.module.css";

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
  showRowsPerPage = true,
  enableColumnResizing = true,
  enableRowDragging = false,
  getRowId,
}: DataTableProps<T>) {
  const { table, columnConfigMap, globalFilter } = useDataTable({
    tableType,
    columns,
    data,
    paginationConfig,
    enableColumnResizing,
    enableRowDragging,
    getRowId,
  });

  const rowDragging = useRowDragging(table);
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
        {renderPaginationControl(
          tableType,
          paginationConfig,
          table,
          showRowsPerPage
        )}
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
          <TableHeader
            headerGroups={table.getHeaderGroups()}
            columnConfigMap={columnConfigMap}
            hoveredColumnId={hoveredColumnId}
            setHoveredColumnId={setHoveredColumnId}
            isResizingColumn={isResizingColumn}
            table={table}
            tableHeaderCellStyle={tableHeaderCellStyle}
            tableFirstHeaderCellStyle={tableFirstHeaderCellStyle}
            tableLastHeaderCellStyle={tableLastHeaderCellStyle}
          />
          <TableBody
            rows={table.getRowModel().rows}
            columnConfigMap={columnConfigMap}
            hoveredColumnId={hoveredColumnId}
            tableDataCellStyle={tableDataCellStyle}
            enableRowDragging={enableRowDragging}
            draggingRowId={rowDragging.draggingRowId}
            onDragStart={rowDragging.handleDragStart}
            onDragEnd={rowDragging.handleDragEnd}
          />
        </table>
      </div>
    </div>
  );
}
