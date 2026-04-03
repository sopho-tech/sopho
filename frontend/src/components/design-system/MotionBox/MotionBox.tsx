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
import { motion, type HTMLMotionProps } from "motion/react";

export type MotionBoxProps = SharedLayoutProps & {
  as?: BoxElement;
  display?: Display;
  children: React.ReactNode;
  sx?: React.CSSProperties;
} & Omit<HTMLMotionProps<"div">, keyof SharedLayoutProps | "style">;

export function MotionBox({
  as = "div",
  backgroundColor = "default",
  display,
  ref,
  children,
  sx,
  ...props
}: MotionBoxProps) {
  const MotionComponent = motion[as];
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
    className
  );

  const mergedStyle = mergeBoxStyles(display, layoutStyles, sx, htmlStyle);

  return (
    <MotionComponent
      // @ts-expect-error - polymorphic motion element ref
      ref={ref}
      className={mergedClassName}
      style={mergedStyle}
      {...htmlPropsWithoutClassNameAndStyle}
    >
      {children}
    </MotionComponent>
  );
}
