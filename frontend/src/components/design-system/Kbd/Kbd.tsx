import React from "react";
import styles from "src/components/design-system/Kbd/Kbd.module.css";

export type KbdSize = "sm" | "md" | "lg";

export type KbdProps = React.HTMLAttributes<HTMLElement> & {
  children: React.ReactNode;
  ref?: React.Ref<HTMLElement>;
};

export const Kbd = ({ children, className, ref, ...restProps }: KbdProps) => {
  const classNames = [styles.kbd, className].filter(Boolean).join(" ");

  return (
    <kbd ref={ref} className={classNames} {...restProps}>
      {children}
    </kbd>
  );
};

export type KbdGroupProps = React.HTMLAttributes<HTMLSpanElement> & {
  children: React.ReactElement<KbdProps>[];
  separator?: string;
  ref?: React.Ref<HTMLSpanElement>;
};

export const KbdGroup = ({
  children,
  separator = "+",
  className,
  ref,
  ...restProps
}: KbdGroupProps) => {
  const classNames = [styles.group, className].filter(Boolean).join(" ");

  return (
    <span ref={ref} className={classNames} {...restProps}>
      {React.Children.map(children, (child, index) => {
        if (index === 0) {
          return child;
        }
        return (
          <React.Fragment key={index}>
            <span className={styles.separator} aria-hidden="true">
              {separator}
            </span>
            {child}
          </React.Fragment>
        );
      })}
    </span>
  );
};
