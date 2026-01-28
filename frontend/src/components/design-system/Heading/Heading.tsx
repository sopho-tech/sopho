import { useMemo } from "react";
import { getCSSVariable } from "src/utils/css_util";

export type FontSize =
  | "xs"
  | "sm"
  | "base"
  | "lg"
  | "xl"
  | "2xl"
  | "3xl"
  | "4xl"
  | "5xl"
  | "6xl";

export type FontWeight = "light" | "normal" | "medium" | "semibold" | "bold";

export type TextColor =
  | "default"
  | "white"
  | "subtle"
  | "disabled"
  | "error"
  | "success"
  | "warning"
  | "black";

type HeadingProps = {
  accessbilityLevel: 1 | 2 | 3 | 4 | 5 | 6;
  weight?: FontWeight;
  size?: FontSize;
  textAlign?: "left" | "center" | "right" | "justify" | "start" | "end";
  textColor?: TextColor;
  children: React.ReactNode;
};

const headingComponents: Record<
  1 | 2 | 3 | 4 | 5 | 6,
  "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
> = {
  1: "h1",
  2: "h2",
  3: "h3",
  4: "h4",
  5: "h5",
  6: "h6",
};

function getFontSize(size: FontSize | undefined): string | undefined {
  if (size === undefined) return undefined;
  return getCSSVariable(`--font-size-${size}`);
}

function getFontWeight(weight: FontWeight | undefined): string | undefined {
  if (weight === undefined) return undefined;
  return getCSSVariable(`--font-weight-${weight}`);
}

function getTextColor(color: TextColor | undefined): string | undefined {
  if (color === undefined) return undefined;

  const colorMap: Record<TextColor, string | undefined> = {
    default: getCSSVariable("--color-text"),
    white: getCSSVariable("--color-background"),
    subtle: getCSSVariable("--color-grey-500"),
    disabled: getCSSVariable("--color-grey-500"),
    error: getCSSVariable("--color-red"),
    success: getCSSVariable("--color-green-dark-2"),
    warning: getCSSVariable("--color-grey-700"),
    black: getCSSVariable("--color-grey-800"),
  };

  return colorMap[color];
}

export function Heading({
  accessbilityLevel,
  children,
  weight,
  size,
  textAlign,
  textColor,
}: HeadingProps) {
  const Component = headingComponents[accessbilityLevel];

  const style = useMemo(() => {
    const styles: React.CSSProperties = {};
    const fontSize = getFontSize(size);
    const fontWeight = getFontWeight(weight);
    const color = getTextColor(textColor);

    if (fontSize !== undefined) {
      styles.fontSize = fontSize;
    }
    if (fontWeight !== undefined) {
      styles.fontWeight = fontWeight;
    }
    if (textAlign !== undefined) {
      styles.textAlign = textAlign;
    }
    if (color !== undefined) {
      styles.color = color;
    }
    return Object.keys(styles).length > 0 ? styles : undefined;
  }, [weight, size, textAlign, textColor]);

  return <Component style={style}>{children}</Component>;
}
