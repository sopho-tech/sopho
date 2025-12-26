import { flexRender, Row } from "@tanstack/react-table";
import { ColumnConfig } from "src/components/design-system/DataTable/types";
import {
  getAlignmentClass,
  getFontClass,
} from "src/components/design-system/DataTable/utils";
import styles from "src/components/design-system/DataTable/DataTable.module.css";

type TableRowProps<T> = {
  row: Row<T>;
  columnConfigMap: Map<string, ColumnConfig<T>>;
  hoveredColumnId: string | null;
  tableDataCellStyle?: string;
  isDragging: boolean;
  enableRowDragging: boolean;
  onDragStart?: (e: React.DragEvent<HTMLTableRowElement>) => void;
  onDragEnd?: () => void;
};

export function TableRow<T>({
  row,
  columnConfigMap,
  hoveredColumnId,
  tableDataCellStyle,
  isDragging,
  enableRowDragging,
  onDragStart,
  onDragEnd,
}: TableRowProps<T>) {
  return (
    <tr
      key={row.id}
      className={`${styles.tableBodyRow} ${isDragging ? styles.tableBodyRowDragging : ""}`}
      draggable={enableRowDragging}
      onDragStart={enableRowDragging ? onDragStart : undefined}
      onDragEnd={enableRowDragging ? onDragEnd : undefined}
    >
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
            {flexRender(cell.column.columnDef.cell, cell.getContext())}
          </td>
        );
      })}
    </tr>
  );
}
