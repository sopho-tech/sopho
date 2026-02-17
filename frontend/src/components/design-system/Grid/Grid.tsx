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
  gutter?: GutterValue;
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

export function Grid({
  as: Component = "div",
  gutter = "sm",
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

  // Build CSS custom properties for responsive gutter
  const style: React.CSSProperties = {};
  let dataGutter: string | undefined;
  let dataResponsive: string | undefined;

  if (isResponsiveGutter(gutter)) {
    const defaultGutter = gutter.base ?? "sm";
    (style as Record<string, string>)["--grid-gutter-default"] = `var(--space-${defaultGutter})`;

    if (gutter.sm !== undefined) {
      (style as Record<string, string>)["--grid-gutter-sm"] = `var(--space-${gutter.sm})`;
    }
    if (gutter.md !== undefined) {
      (style as Record<string, string>)["--grid-gutter-md"] = `var(--space-${gutter.md})`;
    }
    if (gutter.lg !== undefined) {
      (style as Record<string, string>)["--grid-gutter-lg"] = `var(--space-${gutter.lg})`;
    }
    if (gutter.xl !== undefined) {
      (style as Record<string, string>)["--grid-gutter-xl"] = `var(--space-${gutter.xl})`;
    }
    if (gutter["2xl"] !== undefined) {
      (style as Record<string, string>)["--grid-gutter-2xl"] = `var(--space-${gutter["2xl"]})`;
    }
    dataResponsive = "true";
  } else {
    // Simple string value - use data-gutter for backward compatibility
    dataGutter = gutter;
  }

  return (
    <Component
      className={mergedClassName}
      style={style}
      data-gutter={dataGutter}
      data-responsive={dataResponsive}
    >
      {children}
    </Component>
  );
}
