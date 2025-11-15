import { getCSSVariable } from "src/utils/css_util";
import type {
  SharedLayoutProps,
  FlexValue,
  OverflowValue,
  SpacingValue,
  SpacingSize,
  NumberString,
} from "src/components/design-system/datatypes";

export function getFlexStyles(
  flex: FlexValue | undefined
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
  overflow: OverflowValue | undefined
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
  value: SpacingValue | undefined
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
      value.includes("em"))
  ) {
    return value;
  }

  if (typeof value === "string") {
    const spacingSize = value as SpacingSize;
    return getCSSVariable(`--space-${spacingSize}`);
  }

  return value;
}

function getSizeValue(
  value: NumberString | undefined
): string | number | undefined {
  if (value === undefined) return undefined;
  if (typeof value === "number") {
    return `${value}px`;
  }
  return value;
}

export function getSharedLayoutStyles(
  props: SharedLayoutProps
): React.CSSProperties {
  const {
    direction,
    gap,
    paddingX,
    paddingY,
    marginTop,
    marginBottom,
    marginLeft,
    marginRight,
    flex,
    overflow,
    justifyContent,
    alignItems,
    position,
    top,
    bottom,
    left,
    right,
    zIndex,
    height,
    width,
  } = props;

  const gapSize = gap ? getCSSVariable(`--space-${gap}`) : undefined;
  const paddingXSize = paddingX
    ? getCSSVariable(`--space-${paddingX}`)
    : undefined;
  const paddingYSize = paddingY
    ? getCSSVariable(`--space-${paddingY}`)
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
    ...getFlexStyles(flex),
    ...getOverflowStyles(overflow),
  };
}
