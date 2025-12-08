import React from "react";
import { GridItem } from "src/components/design-system/Grid/GridItem";
import styles from "src/components/design-system/Grid/Grid.module.css";

type GutterSize = "sm" | "md" | "lg";

type GridProps = {
  gutter?: GutterSize;
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

export function Grid({ gutter = "sm", children }: GridProps) {
  const childrenArray = React.Children.toArray(children);
  const invalidChildren = childrenArray.filter(
    (child) => !isValidGridItem(child)
  );

  if (invalidChildren.length > 0) {
    throw new Error(
      `Grid component only accepts GridItem as direct children. Found ${invalidChildren.length} invalid child(ren).`
    );
  }

  return (
    <div className={styles.grid} data-gutter={gutter}>
      {children}
    </div>
  );
}
