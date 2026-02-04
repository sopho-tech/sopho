import classNames from "classnames";
import styles from "src/components/design-system/Badge/Badge.module.css";

export type BadgeProps = React.HTMLAttributes<HTMLElement> & {
  variant?: "default" | "subtle" | "blue" | "green" | "yellow";
  children: string | number;
};

export function Badge({
  variant = "default",
  className,
  children,
  ...restProps
}: BadgeProps) {
  return (
    <span
      className={classNames(styles.badge, styles[variant], className)}
      {...restProps}
    >
      {children}
    </span>
  );
}
