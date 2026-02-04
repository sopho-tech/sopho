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
  children: React.ReactNode;
  sx?: React.CSSProperties;
} & Omit<React.HTMLAttributes<HTMLElement>, keyof SharedLayoutProps | "style">;

export function Box({
  as = "div",
  backgroundColor = "default",
  display,
  ref,
  children,
  sx,
  ...props
}: BoxProps) {
  const Component = as;
  const { layoutProps, htmlProps } = separateLayoutProps(props);

  const layoutStyles = getSharedLayoutStyles(layoutProps);

  const backgroundColorClassName = styles[backgroundColor];
  const mergedClassName = mergeBoxClassName(
    backgroundColorClassName,
    htmlProps.className
  );

  const {
    className: _,
    style: htmlStyle,
    ...htmlPropsWithoutClassNameAndStyle
  } = htmlProps;

  const mergedStyle = mergeBoxStyles(display, layoutStyles, sx, htmlStyle);

  return (
    <Component
      ref={ref as any}
      className={mergedClassName}
      style={mergedStyle}
      {...htmlPropsWithoutClassNameAndStyle}
    >
      {children}
    </Component>
  );
}
