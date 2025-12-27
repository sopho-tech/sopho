import { Icon } from "src/components/design-system/Icon";
import React from "react";
import { Text } from "src/components/design-system/Text/Text";
import {
  ButtonSize,
  ButtonType,
  ButtonEmphasis,
  IconType,
} from "src/components/design-system/datatypes";
import styles from "src/components/design-system/Button/Button.module.css";

type ButtonProps = {
  onClick: (event: {
    event:
      | React.MouseEvent<HTMLButtonElement>
      | React.KeyboardEvent<HTMLButtonElement>;
  }) => void;
  leadingIconName?: IconType;
  trailingIconName?: IconType;
  label: string;
  shape: "pill" | "rectangle" | "square" | "circle";
  backgroundColor:
    | "accent"
    | "green"
    | "red"
    | "lightgrey"
    | "grey"
    | "transparent"
    | "white";
  disabled?: boolean;
  fullWidth?: boolean;
  size: ButtonSize;
  type?: ButtonType;
  emphasis?: ButtonEmphasis;
};

function getTextColor(backgroundColor: string) {
  switch (backgroundColor) {
    case "accent":
      return "white";
    case "grey":
      return "black";
    default:
      return "default";
  }
}

export function Button({
  onClick,
  leadingIconName,
  trailingIconName,
  label,
  shape: _shape,
  backgroundColor,
  disabled = false,
  fullWidth = false,
  size,
  type = "button",
  emphasis: _emphasis,
}: ButtonProps) {
  const backgroundColorClassName = styles[backgroundColor];
  const sizeClassName =
    styles[`size${size.charAt(0).toUpperCase() + size.slice(1)}`];
  const fullWidthClassName = fullWidth ? styles.fullWidth : "";
  const textColor = getTextColor(backgroundColor);
  const iconColor = backgroundColor === "white" ? "grey" : "white";
  return (
    <button
      type={type}
      disabled={disabled}
      className={`${styles.button} ${sizeClassName} ${backgroundColorClassName} ${fullWidthClassName}`}
      onClick={(e) => onClick({ event: e })}
    >
      {leadingIconName && (
        <Icon type={leadingIconName} color={iconColor}></Icon>
      )}
      <Text color={textColor}>{label}</Text>
      {trailingIconName && (
        <Icon type={trailingIconName} color={iconColor}></Icon>
      )}
    </button>
  );
}
