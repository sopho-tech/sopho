import { useMemo } from "react";

type HeadingProps = {
  accessbilityLevel: 1 | 2 | 3 | 4 | 5 | 6;
  weight?: number | string;
  size?: number | string;
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

export function Heading({
  accessbilityLevel,
  children,
  weight,
  size,
}: HeadingProps) {
  const Component = headingComponents[accessbilityLevel];

  const style = useMemo(() => {
    const styles: React.CSSProperties = {};
    if (weight !== undefined) {
      styles.fontWeight = weight;
    }
    if (size !== undefined) {
      styles.fontSize = size;
    }
    return Object.keys(styles).length > 0 ? styles : undefined;
  }, [weight, size]);

  return <Component style={style}>{children}</Component>;
}
