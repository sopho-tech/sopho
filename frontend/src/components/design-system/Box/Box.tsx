import styles from "src/components/design-system/Box/Box.module.css";
import type {
  SharedLayoutProps,
  BoxElement,
  Display,
} from "src/components/design-system/datatypes";
import { getSharedLayoutStyles } from "src/components/design-system/utils";

export type BoxProps = SharedLayoutProps & {
  as?: BoxElement;
  display?: Display;
  children: React.ReactNode;
};

export function Box({
  as = "div",
  color = "default",
  display,
  ref,
  children,
  ...layoutProps
}: BoxProps) {
  const Component = as;
  const layoutStyles = getSharedLayoutStyles(layoutProps);

  return (
    <Component
      ref={ref as any}
      className={styles[color]}
      style={{
        display,
        ...layoutStyles,
      }}
    >
      {children}
    </Component>
  );
}
