import { useMemo } from "react";
import styles from "src/components/design-system/Table/Table.module.css";

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

function TableHeaderCell({ children, align = "start" }: TableCellProps) {
  return (
    <th className={`${styles.cell} ${align === "end" ? styles.alignEnd : ""}`}>
      {children}
    </th>
  );
}

type TableRowProps = {
  "aria-label"?: string;
  children: React.ReactNode;
};

function TableRow({ "aria-label": ariaLabel, children }: TableRowProps) {
  return (
    <tr aria-label={ariaLabel} className={styles.row}>
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
