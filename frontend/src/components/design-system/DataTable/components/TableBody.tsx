import { Row } from "@tanstack/react-table";
import { ColumnConfig } from "src/components/design-system/DataTable/types";
import { TableRow } from "src/components/design-system/DataTable/components/TableRow";
import styles from "src/components/design-system/DataTable/DataTable.module.css";

type TableBodyProps<T> = {
  rows: Row<T>[];
  columnConfigMap: Map<string, ColumnConfig<T>>;
  hoveredColumnId: string | null;
  tableDataCellStyle?: string;
  enableRowDragging: boolean;
  draggingRowId: string | null;
  onDragStart: (e: React.DragEvent<HTMLTableRowElement>, row: Row<T>) => void;
  onDragEnd: () => void;
  emptySearchMessage?: string;
  columnCount: number;
};

export function TableBody<T>({
  rows,
  columnConfigMap,
  hoveredColumnId,
  tableDataCellStyle,
  enableRowDragging,
  draggingRowId,
  onDragStart,
  onDragEnd,
  emptySearchMessage,
  columnCount,
}: TableBodyProps<T>) {
  return (
    <tbody className={styles.tableBody}>
      {emptySearchMessage ? (
        <tr>
          <td colSpan={columnCount} className={styles.emptyStateCell}>
            {emptySearchMessage}
          </td>
        </tr>
      ) : (
        rows.map((row) => {
        const isDragging = draggingRowId === row.id;
        return (
          <TableRow
            key={row.id}
            row={row}
            columnConfigMap={columnConfigMap}
            hoveredColumnId={hoveredColumnId}
            tableDataCellStyle={tableDataCellStyle}
            isDragging={isDragging}
            enableRowDragging={enableRowDragging}
            onDragStart={(e) => onDragStart(e, row)}
          onDragEnd={onDragEnd}
        />
        );
      })
      )}
    </tbody>
  );
}

