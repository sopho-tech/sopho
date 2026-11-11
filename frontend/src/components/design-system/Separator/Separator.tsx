import * as RadixSeparator from "@radix-ui/react-separator";
import classNames from "classnames";
import styles from "src/components/design-system/Separator/Separator.module.css";

export type SeparatorProps = React.ComponentPropsWithoutRef<
  typeof RadixSeparator.Root
> & {
  label?: string;
};

export const Separator = ({
  orientation = "horizontal",
  decorative = true,
  className,
  label,
  ...restProps
}: SeparatorProps) => {
  const line = (
    <RadixSeparator.Root
      orientation={orientation}
      decorative={decorative}
      className={classNames(styles.separator, styles[orientation], className)}
      {...restProps}
    />
  );
  if (!label) {
    return line;
  }
  return (
    <div className={styles.labelledSeparator}>
      {line}
      <span className={styles.label}>{label}</span>
      {line}
    </div>
  );
};
