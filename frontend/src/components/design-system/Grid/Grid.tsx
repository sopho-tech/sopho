import React from "react";
import { BreakpointName } from "src/components/design-system/breakpoints";
import { GridItem } from "src/components/design-system/Grid/GridItem";
import styles from "src/components/design-system/Grid/Grid.module.css";

type GutterSize = "sm" | "md" | "lg" | "xl" | "2xl";

type ResponsiveGutter = {
  base?: GutterSize;
} & Partial<Record<BreakpointName, GutterSize>>;

type GutterValue = GutterSize | ResponsiveGutter;

type GridProps = {
  as?: React.ElementType;
  columnGutter?: GutterValue;
  rowGutter?: GutterValue;
  className?: string;
  children: React.ReactNode;
};

function isValidGridItem(child: React.ReactNode): boolean {
  if (!React.isValidElement(child)) {
    return false;
  }

  const componentType = child.type as React.ComponentType<any>;
  return (
    child.type === GridItem ||
    (componentType && componentType.displayName === "GridItem")
  );
}

function isResponsiveGutter(gutter: GutterValue): gutter is ResponsiveGutter {
  return typeof gutter === "object" && gutter !== null;
}

function applyResponsiveVars(
  style: Record<string, string>,
  gutter: ResponsiveGutter,
  prefix: string
) {
  const defaultGutter = gutter.base ?? "sm";
  style[`--grid-${prefix}-gutter-default`] = `var(--space-${defaultGutter})`;

  if (gutter.sm !== undefined) {
    style[`--grid-${prefix}-gutter-sm`] = `var(--space-${gutter.sm})`;
  }
  if (gutter.md !== undefined) {
    style[`--grid-${prefix}-gutter-md`] = `var(--space-${gutter.md})`;
  }
  if (gutter.lg !== undefined) {
    style[`--grid-${prefix}-gutter-lg`] = `var(--space-${gutter.lg})`;
  }
  if (gutter.xl !== undefined) {
    style[`--grid-${prefix}-gutter-xl`] = `var(--space-${gutter.xl})`;
  }
  if (gutter["2xl"] !== undefined) {
    style[`--grid-${prefix}-gutter-2xl`] = `var(--space-${gutter["2xl"]})`;
  }
}

export function Grid({
  as: Component = "div",
  columnGutter = "sm",
  rowGutter = "sm",
  className,
  children,
}: GridProps) {
  const childrenArray = React.Children.toArray(children);
  const invalidChildren = childrenArray.filter(
    (child) => !isValidGridItem(child)
  );

  if (invalidChildren.length > 0) {
    throw new Error(
      `Grid component only accepts GridItem as direct children. Found ${invalidChildren.length} invalid child(ren).`
    );
  }

  const mergedClassName = className
    ? `${styles.grid} ${className}`.trim()
    : styles.grid;

  const style: Record<string, string> = {};
  let dataColumnGutter: string | undefined;
  let dataRowGutter: string | undefined;
  let dataColResponsive: string | undefined;
  let dataRowResponsive: string | undefined;

  if (isResponsiveGutter(columnGutter)) {
    applyResponsiveVars(style, columnGutter, "col");
    dataColResponsive = "true";
  } else {
    dataColumnGutter = columnGutter;
  }

  if (isResponsiveGutter(rowGutter)) {
    applyResponsiveVars(style, rowGutter, "row");
    dataRowResponsive = "true";
  } else {
    dataRowGutter = rowGutter;
  }

  return (
    <Component
      className={mergedClassName}
      style={style}
      data-column-gutter={dataColumnGutter}
      data-row-gutter={dataRowGutter}
      data-col-responsive={dataColResponsive}
      data-row-responsive={dataRowResponsive}
    >
      {children}
    </Component>
  );
}
