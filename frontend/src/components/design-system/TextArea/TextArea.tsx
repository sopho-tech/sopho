import classNames from "classnames";
import { Ref, TextareaHTMLAttributes } from "react";
import styles from "src/components/design-system/TextArea/TextArea.module.css";

export type TextAreaProps = {
  value: string;
  onChange: (event: React.ChangeEvent<HTMLTextAreaElement>) => void;
  rows?: number;
  className?: string;
  ref?: Ref<HTMLTextAreaElement>;
} & Omit<
  TextareaHTMLAttributes<HTMLTextAreaElement>,
  "value" | "onChange" | "rows" | "className" | "ref"
>;

export function TextArea({
  value,
  onChange,
  rows = 4,
  className,
  ref,
  ...otherProps
}: TextAreaProps) {
  return (
    <textarea
      ref={ref}
      value={value}
      onChange={onChange}
      rows={rows}
      className={classNames(styles.textArea, className)}
      {...otherProps}
    />
  );
}
