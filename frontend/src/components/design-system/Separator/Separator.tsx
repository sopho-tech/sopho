import * as RadixSeparator from "@radix-ui/react-separator";
import classNames from "classnames";
import styles from "./Separator.module.css";

export type SeparatorProps = React.ComponentPropsWithoutRef<
  typeof RadixSeparator.Root
>;

export const Separator = ({
  orientation = "horizontal",
  decorative = true,
  className,
  ...restProps
}: SeparatorProps) => (
  <RadixSeparator.Root
    orientation={orientation}
    decorative={decorative}
    className={classNames(styles.separator, styles[orientation], className)}
    {...restProps}
  />
);
