import styles from "src/components/design-system/Box/Box.module.css";
import type {
  SharedLayoutProps,
  BoxElement,
  Display,
} from "src/components/design-system/datatypes";
import { getSharedLayoutStyles } from "src/components/design-system/utils";

const layoutPropKeys: (keyof SharedLayoutProps)[] = [
  "borderRadius",
  "border",
  "shadow",
  "color",
  "direction",
  "gap",
  "paddingX",
  "paddingY",
  "marginTop",
  "marginBottom",
  "marginLeft",
  "marginRight",
  "flex",
  "overflow",
  "justifyContent",
  "alignItems",
  "alignContent",
  "top",
  "bottom",
  "left",
  "right",
  "position",
  "zIndex",
  "height",
  "width",
  "ref",
];

export type BoxProps = SharedLayoutProps & {
  as?: BoxElement;
  display?: Display;
  children: React.ReactNode;
  sx?: React.CSSProperties;
} & Omit<React.HTMLAttributes<HTMLElement>, keyof SharedLayoutProps | "style">;

export function Box({
  as = "div",
  color = "default",
  display,
  ref,
  children,
  sx,
  ...props
}: BoxProps) {
  const Component = as;
  const layoutProps: SharedLayoutProps = {};
  const htmlProps: React.HTMLAttributes<HTMLElement> = {};

  Object.entries(props).forEach(([key, value]) => {
    if (layoutPropKeys.includes(key as keyof SharedLayoutProps)) {
      (layoutProps as any)[key] = value;
    } else {
      (htmlProps as any)[key] = value;
    }
  });

  const layoutStyles = getSharedLayoutStyles(layoutProps);

  return (
    <Component
      ref={ref as any}
      className={styles[color]}
      style={{
        display,
        ...layoutStyles,
        ...sx,
      }}
      {...htmlProps}
    >
      {children}
    </Component>
  );
}
