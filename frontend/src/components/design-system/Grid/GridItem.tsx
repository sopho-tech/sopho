import React from "react";
import styles from "src/components/design-system/Grid/GridItem.module.css";

type GridItemProps = {
  colSpan?: number;
  rowSpan?: number;
  children: React.ReactNode;
};

export function GridItem({ colSpan = 12, rowSpan = 1, children }: GridItemProps) {
  if (colSpan < 1 || colSpan > 12) {
    throw new Error(
      `GridItem colSpan must be between 1 and 12, received ${colSpan}`
    );
  }

  if (rowSpan < 1) {
    throw new Error(`GridItem rowSpan must be at least 1, received ${rowSpan}`);
  }

  return (
    <div
      className={styles.gridItem}
      style={{
        gridColumn: `span ${colSpan}`,
        gridRow: `span ${rowSpan}`,
      }}
    >
      {children}
    </div>
  );
}

GridItem.displayName = "GridItem";
