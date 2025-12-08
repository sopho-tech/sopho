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

type HeadingProps = {
  accessbilityLevel: 1 | 2 | 3 | 4 | 5 | 6;
  weight?: FontWeight;
  size?: FontSize;
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

export function Heading({
  accessbilityLevel,
  children,
  weight,
  size,
}: HeadingProps) {
  const Component = headingComponents[accessbilityLevel];

  const style = useMemo(() => {
    const styles: React.CSSProperties = {};
    const fontSize = getFontSize(size);
    const fontWeight = getFontWeight(weight);

    if (fontSize !== undefined) {
      styles.fontSize = fontSize;
    }
    if (fontWeight !== undefined) {
      styles.fontWeight = fontWeight;
    }
    return Object.keys(styles).length > 0 ? styles : undefined;
  }, [weight, size]);

  return <Component style={style}>{children}</Component>;
}
