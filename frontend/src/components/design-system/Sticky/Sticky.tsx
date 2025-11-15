import React from "react";
import type { SharedLayoutProps } from "src/components/design-system/datatypes";
import { getSharedLayoutStyles } from "src/components/design-system/utils";

export type StickyProps = SharedLayoutProps & {
  children: React.ReactElement;
};

export function Sticky({ children, ref, ...layoutProps }: StickyProps) {
  const stickyStyles = getSharedLayoutStyles({
    ...layoutProps,
    position: "sticky",
  });

  const existingStyle = (children.props as { style?: React.CSSProperties })
    ?.style;

  return React.cloneElement(children, {
    ...(children.props as object),
    style: {
      ...existingStyle,
      ...stickyStyles,
      position: "sticky",
    },
    ref: ref,
  } as any);
}
