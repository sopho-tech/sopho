// TODO Fix @ts-expect-error - polymorphic Component expects intersection of all element refs
import styles from "src/components/design-system/Box/Box.module.css";
import type {
  SharedLayoutProps,
  BoxElement,
  Display,
} from "src/components/design-system/datatypes";
import {
  getSharedLayoutStyles,
  separateLayoutProps,
  mergeBoxStyles,
  mergeBoxClassName,
} from "src/components/design-system/utils";

export type BoxProps = SharedLayoutProps & {
  as?: BoxElement;
  display?: Display;
  children?: React.ReactNode;
  sx?: React.CSSProperties;
  /** Marks this element as the host whose hover or focus reveals its chrome. */
  revealChildrenOnHover?: boolean;
  /** Hides this element until an ancestor host is hovered or focused within. */
  revealOnHover?: boolean;
} & Omit<React.HTMLAttributes<HTMLElement>, keyof SharedLayoutProps | "style">;

export function Box({
  as = "div",
  backgroundColor = "default",
  display,
  ref,
  children,
  sx,
  revealChildrenOnHover = false,
  revealOnHover = false,
  ...props
}: BoxProps) {
  const Component = as;
  const { layoutProps, htmlProps } = separateLayoutProps(props);

  const layoutStyles = getSharedLayoutStyles(layoutProps);

  const backgroundColorClassName = styles[backgroundColor];
  const {
    className,
    style: htmlStyle,
    ...htmlPropsWithoutClassNameAndStyle
  } = htmlProps;

  const mergedClassName = mergeBoxClassName(
    backgroundColorClassName,
    revealChildrenOnHover && styles.revealChildrenOnHover,
    revealOnHover && styles.revealOnHover,
    className,
  );

  const mergedStyle = mergeBoxStyles(display, layoutStyles, sx, htmlStyle);

  return (
    <Component
      // @ts-expect-error - polymorphic Component expects intersection of all element refs
      ref={ref}
      className={mergedClassName}
      style={mergedStyle}
      {...htmlPropsWithoutClassNameAndStyle}
    >
      {children}
    </Component>
  );
}
