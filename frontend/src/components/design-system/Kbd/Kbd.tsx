import React from "react";
import styles from "src/components/design-system/Kbd/Kbd.module.css";

export type KbdProps = React.HTMLAttributes<HTMLElement> & {
  children: React.ReactNode;
};

export const Kbd = React.forwardRef<HTMLElement, KbdProps>(
  ({ children, className, ...restProps }, ref) => {
    const classNames = [styles.kbd, className].filter(Boolean).join(" ");

    return (
      <kbd ref={ref} className={classNames} {...restProps}>
        {children}
      </kbd>
    );
  }
);

Kbd.displayName = "Kbd";

export type KbdGroupProps = React.HTMLAttributes<HTMLSpanElement> & {
  children: React.ReactElement<KbdProps>[];
  separator?: string;
};

export const KbdGroup = React.forwardRef<HTMLSpanElement, KbdGroupProps>(
  ({ children, separator = "+", className, ...restProps }, ref) => {
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
  }
);

KbdGroup.displayName = "KbdGroup";
