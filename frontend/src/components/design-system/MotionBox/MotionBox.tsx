import styles from "src/components/design-system/Box/Box.module.css";
import type {
  SharedLayoutProps,
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
  display?: Display;
  children: React.ReactNode;
  sx?: React.CSSProperties;
} & Omit<HTMLMotionProps<"div">, keyof SharedLayoutProps | "style">;

export function MotionBox({
  backgroundColor = "default",
  display,
  ref,
  children,
  sx,
  ...props
}: MotionBoxProps) {
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
    <motion.div
      ref={ref as any}
      className={mergedClassName}
      style={mergedStyle}
      {...htmlPropsWithoutClassNameAndStyle}
    >
      {children}
    </motion.div>
  );
}
