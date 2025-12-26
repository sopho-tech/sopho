import { flexRender, Table, HeaderGroup } from "@tanstack/react-table";
import { ColumnConfig } from "src/components/design-system/DataTable/types";
import { getHeaderAlignmentClass, getHeaderJustifyContent } from "src/components/design-system/DataTable/utils";
import { Icon } from "src/components/design-system/Icon/Icon";
import { Flex } from "src/components/design-system/Flex";
import styles from "src/components/design-system/DataTable/DataTable.module.css";

type TableHeaderProps<T> = {
  headerGroups: HeaderGroup<T>[];
  columnConfigMap: Map<string, ColumnConfig<T>>;
  hoveredColumnId: string | null;
  setHoveredColumnId: (id: string | null) => void;
  isResizingColumn: boolean;
  table: Table<T>;
  tableHeaderCellStyle?: string;
  tableFirstHeaderCellStyle?: string;
  tableLastHeaderCellStyle?: string;
};

export function TableHeader<T>({
  headerGroups,
  columnConfigMap,
  hoveredColumnId,
  setHoveredColumnId,
  isResizingColumn,
  table,
  tableHeaderCellStyle,
  tableFirstHeaderCellStyle,
  tableLastHeaderCellStyle,
}: TableHeaderProps<T>) {
  return (
    <thead className={styles.thead}>
      {headerGroups.map((headerGroup) => (
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
  );
}

