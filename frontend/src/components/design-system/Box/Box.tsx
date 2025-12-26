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
  "backgroundColor",
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
  "alignSelf",
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
  backgroundColor = "default",
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

  const backgroundColorClassName = styles[backgroundColor];
  const mergedClassName = htmlProps.className
    ? `${backgroundColorClassName} ${htmlProps.className}`.trim()
    : backgroundColorClassName;

  const {
    className: _,
    style: htmlStyle,
    ...htmlPropsWithoutClassNameAndStyle
  } = htmlProps;

  const mergedStyle = {
    display,
    ...layoutStyles,
    ...sx,
    ...htmlStyle,
  };

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
