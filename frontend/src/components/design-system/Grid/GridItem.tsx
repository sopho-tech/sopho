import React from "react";
import { BreakpointName } from "src/components/design-system/breakpoints.tsx";
import styles from "src/components/design-system/Grid/GridItem.module.css";

type ResponsiveColSpan = {
  base?: number;
} & Partial<Record<BreakpointName, number>>;

type ColSpanValue = number | ResponsiveColSpan;

type GridItemProps = {
  as?: React.ElementType;
  colSpan?: ColSpanValue;
  rowSpan?: number;
  className?: string;
  children: React.ReactNode;
};

function validateColSpan(value: number): void {
  if (value < 1 || value > 12) {
    throw new Error(
      `GridItem colSpan must be between 1 and 12, received ${value}`
    );
  }
}

function isResponsiveColSpan(
  colSpan: ColSpanValue
): colSpan is ResponsiveColSpan {
  return typeof colSpan === "object" && colSpan !== null;
}

export function GridItem({
  as: Component = "div",
  colSpan = 12,
  rowSpan = 1,
  className,
  children,
}: GridItemProps) {
  // Validate colSpan values
  if (isResponsiveColSpan(colSpan)) {
    // Validate all breakpoint values
    Object.values(colSpan).forEach((value) => {
      if (typeof value === "number") {
        validateColSpan(value);
      }
    });
  } else {
    validateColSpan(colSpan);
  }

  if (rowSpan < 1) {
    throw new Error(`GridItem rowSpan must be at least 1, received ${rowSpan}`);
  }

  const mergedClassName = className
    ? `${styles.gridItem} ${className}`.trim()
    : styles.gridItem;

  // Build CSS custom properties for responsive colSpan
  const style: React.CSSProperties = {
    gridRow: `span ${rowSpan}`,
  };

  let dataResponsive: string | undefined;

  if (isResponsiveColSpan(colSpan)) {
    // Set CSS custom properties for each breakpoint
    const defaultSpan = colSpan.base ?? 12;
    style["--grid-col-span-default" as any] = defaultSpan;

    // Set breakpoint-specific values
    if (colSpan.sm !== undefined) {
      style["--grid-col-span-sm" as any] = colSpan.sm;
    }
    if (colSpan.md !== undefined) {
      style["--grid-col-span-md" as any] = colSpan.md;
    }
    if (colSpan.lg !== undefined) {
      style["--grid-col-span-lg" as any] = colSpan.lg;
    }
    if (colSpan.xl !== undefined) {
      style["--grid-col-span-xl" as any] = colSpan.xl;
    }
    if (colSpan["2xl"] !== undefined) {
      style["--grid-col-span-2xl" as any] = colSpan["2xl"];
    }
    dataResponsive = "true";
  } else {
    // Simple number value - use inline style for backward compatibility
    style.gridColumn = `span ${colSpan}`;
  }

  return (
    <Component
      className={mergedClassName}
      style={style}
      data-responsive={dataResponsive}
    >
      {children}
    </Component>
  );
}

GridItem.displayName = "GridItem";
