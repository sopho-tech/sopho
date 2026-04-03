import { LoaderCircle } from "lucide-react";
import { IconColor, IconSize } from "src/components/design-system/datatypes";
import styles from "src/components/design-system/Spinner/Spinner.module.css";

const sizeMap: Record<IconSize, number> = {
  sm: 14,
  md: 20,
  lg: 24,
  "2xl": 450,
};

type SpinnerProps = {
  size?: IconSize;
  color?: IconColor;
  className?: string;
};

export function Spinner({
  size = "md",
  color = "grey",
  className,
}: SpinnerProps) {
  const colorClassName = styles[color] ?? "";
  const iconSize = sizeMap[size];

  return (
    <LoaderCircle
      role="status"
      aria-label="Loading"
      size={iconSize}
      strokeWidth={2.25}
      className={`${styles.spinner} ${colorClassName} ${className ?? ""}`.trim()}
    />
  );
}
