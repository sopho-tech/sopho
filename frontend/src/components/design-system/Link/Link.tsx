import { Link as RouterLink } from "react-router";
import styles from "src/components/design-system/Link/Link.module.css";

type LinkFontSize = "xs" | "sm" | "base";

type LinkProps = {
  to: string;
  fontSize?: LinkFontSize;
  truncate?: boolean;
  children: React.ReactNode;
};

const fontSizeClassNames: Record<LinkFontSize, string> = {
  xs: styles.fontSizeXs,
  sm: styles.fontSizeSm,
  base: styles.fontSizeBase,
};

export function Link({
  to,
  fontSize = "sm",
  truncate = false,
  children,
}: LinkProps) {
  const className = [
    styles.link,
    fontSizeClassNames[fontSize],
    truncate && styles.truncate,
  ]
    .filter(Boolean)
    .join(" ");
  return (
    <RouterLink to={to} className={className}>
      {children}
    </RouterLink>
  );
}
