import classNames from "classnames";
import styles from "src/components/design-system/Badge/Badge.module.css";

export type BadgeProps = React.HTMLAttributes<HTMLElement> & {
  variant?: "default" | "subtle" | "blue" | "green" | "yellow" | "command";
  shape?: "pill" | "rounded";
  size?: "sm" | "md";
  children: string | number;
};

export function Badge({
  variant = "default",
  shape = "pill",
  size = "sm",
  className,
  children,
  ...restProps
}: BadgeProps) {
  return (
    <span
      className={classNames(
        styles.badge,
        styles[variant],
        styles[shape],
        styles[size],
        className,
      )}
      {...restProps}
    >
      {children}
    </span>
  );
}
