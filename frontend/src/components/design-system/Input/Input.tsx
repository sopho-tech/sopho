import { InputHTMLAttributes, Ref } from "react";
import styles from "./Input.module.css";
import { Icon } from "src/components/design-system/Icon";
import { IconButton } from "src/components/design-system/IconButton";
import { IconType } from "src/components/design-system/datatypes";

export type InputProps = {
  value: string | number;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  type?: string;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
  ref?: Ref<HTMLInputElement>;
  leadingIcon?: IconType;
} & Omit<
  InputHTMLAttributes<HTMLInputElement>,
  | "value"
  | "onChange"
  | "type"
  | "placeholder"
  | "disabled"
  | "className"
  | "ref"
>;

export function Input({
  value,
  onChange,
  type = "text",
  placeholder,
  disabled,
  className,
  ref,
  leadingIcon,
  ...otherProps
}: InputProps) {
  const hasValue = value !== "" && value !== null && value !== undefined;

  const handleClear = () => {
    const syntheticEvent = {
      target: { value: "" },
    } as React.ChangeEvent<HTMLInputElement>;
    onChange(syntheticEvent);
  };

  return (
    <div className={styles.container}>
      {leadingIcon && (
        <div className={styles.leadingIcon}>
          <Icon type={leadingIcon} color="grey" size="sm" />
        </div>
      )}
      <input
        ref={ref}
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        disabled={disabled}
        className={`${styles.input} ${leadingIcon ? styles.inputWithLeadingIcon : ""} ${className || ""}`}
        {...otherProps}
      />
      <div
        className={`${styles.clearButton} ${!hasValue ? styles.clearButtonHidden : ""}`}
      >
        <IconButton
          type="close"
          backgroundColor="transparent"
          iconColor="grey"
          iconSize="sm"
          onClick={handleClear}
        />
      </div>
    </div>
  );
}
