import classNames from "classnames";
import { InputHTMLAttributes, Ref, useState } from "react";
import { Icon } from "src/components/design-system/Icon";
import { IconButton } from "src/components/design-system/IconButton";
import { IconSize, IconType } from "src/components/design-system/datatypes";
import styles from "./Input.module.css";

export type InputProps = {
  value: string | number;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  type?: string;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
  ref?: Ref<HTMLInputElement>;
  leadingIcon?: IconType;
  laggingElement?: React.ReactNode;
  clearButtonSize?: IconSize;
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
  laggingElement,
  clearButtonSize,
  ...otherProps
}: InputProps) {
  const [isFocused, setIsFocused] = useState(false);
  const { onFocus: onFocusProp, onBlur: onBlurProp, ...restProps } = otherProps;

  const hasValue = (() => {
    if (value === null || value === undefined) {
      return false;
    }
    if (Array.isArray(value)) {
      return value.length > 0;
    }
    if (typeof value === "string") {
      return value.trim() !== "";
    }
    if (typeof value === "number") {
      return true;
    }
    return Boolean(value);
  })();

  const handleClear = () => {
    const syntheticEvent = {
      target: { value: "" },
    } as React.ChangeEvent<HTMLInputElement>;
    onChange(syntheticEvent);
  };

  const handleClearMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
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
        onFocus={(e) => {
          setIsFocused(true);
          onFocusProp?.(e);
        }}
        onBlur={(e) => {
          setIsFocused(false);
          onBlurProp?.(e);
        }}
        placeholder={placeholder}
        disabled={disabled}
        className={classNames(
          styles.input,
          {
            [styles.inputWithLeadingIcon]: leadingIcon,
            [styles.inputWithLaggingElement]: laggingElement,
          },
          className
        )}
        {...restProps}
      />
      {laggingElement && !hasValue && (
        <div className={styles.laggingElement}>{laggingElement}</div>
      )}
      {!disabled && (
        <div
          className={`${styles.clearButton} ${!hasValue || !isFocused ? styles.clearButtonHidden : ""}`}
          onMouseDown={handleClearMouseDown}
        >
          <IconButton
            type="close"
            backgroundColor="transparent"
            iconColor="grey"
            iconSize={clearButtonSize ?? "sm"}
            onClick={handleClear}
            tabIndex={-1}
          />
        </div>
      )}
    </div>
  );
}
