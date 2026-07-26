import { getCSSVariable } from "src/utils/css_util";
import type {
  SharedLayoutProps,
  FlexValue,
  OverflowValue,
  SpacingValue,
  SpacingSize,
  NumberString,
  BorderVariant,
  ShadowVariant,
} from "src/components/design-system/datatypes";

export const layoutPropKeys: (keyof SharedLayoutProps)[] = [
  "borderRadius",
  "border",
  "borderTop",
  "borderRight",
  "borderBottom",
  "borderLeft",
  "shadow",
  "backgroundColor",
  "direction",
  "gap",
  "paddingX",
  "paddingY",
  "paddingTop",
  "paddingRight",
  "paddingBottom",
  "paddingLeft",
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

export function separateLayoutProps<T extends Record<string, any>>(
  props: T,
): {
  layoutProps: SharedLayoutProps;
  htmlProps: Omit<T, keyof SharedLayoutProps> & {
    className?: string;
    style?: React.CSSProperties;
  };
} {
  const layoutProps: SharedLayoutProps = {};
  const htmlProps: any = {};

  Object.entries(props).forEach(([key, value]) => {
    if (layoutPropKeys.includes(key as keyof SharedLayoutProps)) {
      (layoutProps as any)[key] = value;
    } else {
      htmlProps[key] = value;
    }
  });

  return { layoutProps, htmlProps };
}

export function mergeBoxStyles(
  display: string | undefined,
  layoutStyles: React.CSSProperties,
  sx: React.CSSProperties | undefined,
  htmlStyle: React.CSSProperties | undefined,
): React.CSSProperties {
  return {
    display,
    ...layoutStyles,
    ...sx,
    ...htmlStyle,
  };
}

export function mergeBoxClassName(
  backgroundColorClassName: string,
  htmlClassName?: string,
): string {
  return htmlClassName
    ? `${backgroundColorClassName} ${htmlClassName}`.trim()
    : backgroundColorClassName;
}

export function getFlexStyles(
  flex: FlexValue | undefined,
): React.CSSProperties {
  if (flex === undefined) return {};

  if (flex === "grow") {
    return { flex: "1 1 auto" };
  }
  if (flex === "shrink") {
    return { flex: "0 1 auto" };
  }
  if (flex === "none") {
    return { flex: "none" };
  }

  if (typeof flex === "number") {
    return { flex: flex.toString() };
  }

  if (typeof flex === "string") {
    const numValue = Number(flex);
    if (!isNaN(numValue)) {
      return { flex: numValue.toString() };
    }
  }

  return {};
}

export function getOverflowStyles(
  overflow: OverflowValue | undefined,
): React.CSSProperties {
  if (overflow === "scrollY") {
    return { overflowY: "auto" };
  }
  if (overflow === "scrollX") {
    return { overflowX: "auto" };
  }
  return { overflow: overflow };
}

function getSpacingValue(
  value: SpacingValue | undefined,
): string | number | undefined {
  if (value === undefined) return undefined;

  if (typeof value === "string" && value.startsWith("-")) {
    const spacingSize = value.slice(1) as SpacingSize;
    return `calc(-1 * var(--space-${spacingSize}))`;
  }

  if (
    typeof value === "string" &&
    (value.includes("var(") ||
      value.includes("calc(") ||
      value.includes("px") ||
      value.includes("rem") ||
      value.includes("em") ||
      value.includes("%"))
  ) {
    return value;
  }

  if (typeof value === "string") {
    const spacingSize = value as SpacingSize;
    return getCSSVariable(`--space-${spacingSize}`);
  }

  return value;
}

function getSizeValue(value: NumberString | undefined): string | undefined {
  if (value === undefined) return undefined;
  if (typeof value === "number") {
    return `${value}px`;
  }
  return value;
}

function getBorderRadius(borderRadius: string) {
  return getCSSVariable(`--border-radius-${borderRadius}`);
}

const borderValueByVariant: Record<BorderVariant, string> = {
  default: "var(--border-default-medium)",
  divider: "var(--border-divider)",
};

function getBorderStyles(props: {
  border?: BorderVariant;
  borderTop?: BorderVariant;
  borderRight?: BorderVariant;
  borderBottom?: BorderVariant;
  borderLeft?: BorderVariant;
}): React.CSSProperties {
  const { border, borderTop, borderRight, borderBottom, borderLeft } = props;
  const styles: React.CSSProperties = {};
  if (border !== undefined) {
    const v = borderValueByVariant[border];
    if (v) styles.border = v;
  }
  if (borderTop !== undefined) {
    const v = borderValueByVariant[borderTop];
    if (v) styles.borderTop = v;
  }
  if (borderRight !== undefined) {
    const v = borderValueByVariant[borderRight];
    if (v) styles.borderRight = v;
  }
  if (borderBottom !== undefined) {
    const v = borderValueByVariant[borderBottom];
    if (v) styles.borderBottom = v;
  }
  if (borderLeft !== undefined) {
    const v = borderValueByVariant[borderLeft];
    if (v) styles.borderLeft = v;
  }
  return styles;
}

function getShadowStyles(
  shadow: ShadowVariant | undefined,
): React.CSSProperties {
  if (shadow === undefined) return {};

  const shadowMapping: Record<ShadowVariant, string> = {
    "2xs": "var(--shadow-2xs)",
    xs: "var(--shadow-xs)",
    sm: "var(--shadow-sm)",
    md: "var(--shadow-md)",
    lg: "var(--shadow-lg)",
    xl: "var(--shadow-xl)",
    scroll: "var(--shadow-scroll)",
  };

  const shadowValue = shadowMapping[shadow];
  if (!shadowValue) return {};

  return { boxShadow: shadowValue };
}

export function getSharedLayoutStyles(
  props: SharedLayoutProps,
): React.CSSProperties {
  const {
    direction,
    gap,
    paddingX,
    paddingY,
    paddingTop,
    paddingRight,
    paddingBottom,
    paddingLeft,
    marginTop,
    marginBottom,
    marginLeft,
    marginRight,
    flex,
    overflow,
    justifyContent,
    alignItems,
    alignContent,
    alignSelf,
    position,
    top,
    bottom,
    left,
    right,
    zIndex,
    height,
    width,
    borderRadius,
    border,
    borderTop,
    borderRight,
    borderBottom,
    borderLeft,
    shadow,
  } = props;

  const gapSize = gap ? getCSSVariable(`--space-${gap}`) : undefined;
  const paddingXSize = paddingX
    ? getCSSVariable(`--space-${paddingX}`)
    : undefined;
  const paddingYSize = paddingY
    ? getCSSVariable(`--space-${paddingY}`)
    : undefined;
  const paddingTopSize = paddingTop
    ? getCSSVariable(`--space-${paddingTop}`)
    : undefined;
  const paddingRightSize = paddingRight
    ? getCSSVariable(`--space-${paddingRight}`)
    : undefined;
  const paddingBottomSize = paddingBottom
    ? getCSSVariable(`--space-${paddingBottom}`)
    : undefined;
  const paddingLeftSize = paddingLeft
    ? getCSSVariable(`--space-${paddingLeft}`)
    : undefined;
  const zIndexVal = zIndex ? getCSSVariable(`--z-index-${zIndex}`) : undefined;

  return {
    ...(position && { position }),
    ...(top !== undefined && { top: getSpacingValue(top) }),
    ...(bottom !== undefined && { bottom: getSpacingValue(bottom) }),
    ...(left !== undefined && { left: getSpacingValue(left) }),
    ...(right !== undefined && { right: getSpacingValue(right) }),
    ...(zIndexVal && { zIndex: zIndexVal }),
    ...(direction && { flexDirection: direction }),
    ...(gapSize && { gap: gapSize }),
    ...(paddingXSize && {
      paddingLeft: paddingXSize,
      paddingRight: paddingXSize,
    }),
    ...(paddingYSize && {
      paddingTop: paddingYSize,
      paddingBottom: paddingYSize,
    }),
    ...(paddingTopSize && { paddingTop: paddingTopSize }),
    ...(paddingRightSize && { paddingRight: paddingRightSize }),
    ...(paddingBottomSize && { paddingBottom: paddingBottomSize }),
    ...(paddingLeftSize && { paddingLeft: paddingLeftSize }),
    ...(marginTop !== undefined && { marginTop: getSpacingValue(marginTop) }),
    ...(marginBottom !== undefined && {
      marginBottom: getSpacingValue(marginBottom),
    }),
    ...(marginLeft !== undefined && {
      marginLeft: getSpacingValue(marginLeft),
    }),
    ...(marginRight !== undefined && {
      marginRight: getSpacingValue(marginRight),
    }),
    ...(height !== undefined && { height: getSizeValue(height) }),
    ...(width !== undefined && { width: getSizeValue(width) }),
    ...(justifyContent && { justifyContent }),
    ...(alignItems && { alignItems }),
    ...(alignContent && { alignContent }),
    ...(alignSelf && { alignSelf }),
    ...(borderRadius && { borderRadius: getBorderRadius(borderRadius) }),
    ...getBorderStyles({
      border,
      borderTop,
      borderRight,
      borderBottom,
      borderLeft,
    }),
    ...getShadowStyles(shadow),
    ...getFlexStyles(flex),
    ...getOverflowStyles(overflow),
  };
}
