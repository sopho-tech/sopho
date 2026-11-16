import { useCallback, useMemo } from "react";
import { Icon } from "src/components/design-system/Icon";
import styles from "src/components/design-system/Table/Table.module.css";

export type TableSortDirection = "asc" | "desc";

const ARIA_SORT_VALUES: Record<TableSortDirection, "ascending" | "descending"> =
  {
    asc: "ascending",
    desc: "descending",
  };
const SORT_ICON_TYPES = {
  asc: "arrow_up",
  desc: "arrow_down",
} as const;

type TableProps = {
  columns: string;
  "aria-label": string;
  children: React.ReactNode;
};

export function Table({
  columns,
  "aria-label": ariaLabel,
  children,
}: TableProps) {
  const columnsStyle = useMemo(
    () => ({ "--table-columns": columns }) as React.CSSProperties,
    [columns],
  );
  return (
    <table aria-label={ariaLabel} className={styles.table} style={columnsStyle}>
      <tbody className={styles.body}>{children}</tbody>
    </table>
  );
}

type TableHeaderProps = {
  children: React.ReactNode;
};

function TableHeader({ children }: TableHeaderProps) {
  return <tr className={`${styles.row} ${styles.headerRow}`}>{children}</tr>;
}

type TableCellProps = {
  children?: React.ReactNode;
  align?: "start" | "end";
};

type TableHeaderCellProps = TableCellProps & {
  onSort?: () => void;
  sortDirection?: TableSortDirection;
};

function TableHeaderCell({
  children,
  align = "start",
  onSort,
  sortDirection,
}: TableHeaderCellProps) {
  const className = `${styles.cell} ${align === "end" ? styles.alignEnd : ""}`;
  if (!onSort) {
    return (
      <th scope="col" className={className}>
        {children}
      </th>
    );
  }
  return (
    <th
      scope="col"
      className={className}
      aria-sort={sortDirection ? ARIA_SORT_VALUES[sortDirection] : "none"}
    >
      <button
        type="button"
        className={`${styles.sortButton} ${sortDirection ? styles.sortButtonActive : ""}`}
        onClick={onSort}
      >
        {children}
        <Icon
          type={sortDirection ? SORT_ICON_TYPES[sortDirection] : "swap_vert"}
          size="sm"
          color="grey"
        />
      </button>
    </th>
  );
}

type TableRowProps = {
  "aria-label"?: string;
  onClick?: () => void;
  children: React.ReactNode;
};

const ROW_ACTIVATION_KEYS = ["Enter", " "];

function TableRow({
  "aria-label": ariaLabel,
  onClick,
  children,
}: TableRowProps) {
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLTableRowElement>) => {
      if (onClick && ROW_ACTIVATION_KEYS.includes(event.key)) {
        event.preventDefault();
        onClick();
      }
    },
    [onClick],
  );
  if (!onClick) {
    return (
      <tr aria-label={ariaLabel} className={styles.row}>
        {children}
      </tr>
    );
  }
  return (
    <tr
      aria-label={ariaLabel}
      className={`${styles.row} ${styles.clickableRow}`}
      tabIndex={0}
      onClick={onClick}
      onKeyDown={handleKeyDown}
    >
      {children}
    </tr>
  );
}

function TableCell({ children, align = "start" }: TableCellProps) {
  return (
    <td className={`${styles.cell} ${align === "end" ? styles.alignEnd : ""}`}>
      {children}
    </td>
  );
}

Table.Header = TableHeader;
Table.HeaderCell = TableHeaderCell;
Table.Row = TableRow;
Table.Cell = TableCell;
