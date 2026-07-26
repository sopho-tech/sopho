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
import { motion, type HTMLMotionProps } from "motion/react";
import { Scale, Duration, EASE } from "src/components/design-system/animation";

type ButtonProps = Omit<
  React.ButtonHTMLAttributes<HTMLButtonElement>,
  keyof HTMLMotionProps<"button">
> &
  Omit<HTMLMotionProps<"button">, "children"> & {
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
      | "ghost"
      | "white";
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
    case "red":
      return "white";
    case "ghost":
      return "subtle";
    default:
      return "default";
  }
}

const buttonSizeToFontSize = {
  sm: "sm",
  md: "base",
  lg: "base",
} as const;

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      onClick,
      leadingIconName,
      trailingIconName,
      label,
      shape,
      backgroundColor,
      disabled = false,
      fullWidth = false,
      size,
      type = "button",
      emphasis,
      ...restProps
    },
    ref,
  ) => {
    const backgroundColorClassName = styles[backgroundColor];
    const sizeClassName =
      styles[`size${size.charAt(0).toUpperCase() + size.slice(1)}`];
    const fullWidthClassName = fullWidth ? styles.fullWidth : "";
    const textColor = getTextColor(backgroundColor);
    const iconColor =
      backgroundColor === "white" || backgroundColor === "ghost"
        ? "grey"
        : "white";

    return (
      <motion.button
        {...restProps}
        ref={ref}
        type={type}
        disabled={disabled}
        data-shape={shape}
        data-emphasis={emphasis}
        className={`${styles.button} ${sizeClassName} ${backgroundColorClassName} ${fullWidthClassName}`}
        onClick={onClick}
        whileHover={disabled ? undefined : { scale: Scale.HOVER }}
        whileTap={disabled ? undefined : { scale: Scale.TAP }}
        transition={{
          duration: Duration.FAST,
          ease: EASE,
        }}
      >
        {leadingIconName && (
          <Icon type={leadingIconName} color={iconColor}></Icon>
        )}
        <Text color={textColor} fontSize={buttonSizeToFontSize[size]}>
          {label}
        </Text>
        {trailingIconName && (
          <Icon type={trailingIconName} color={iconColor}></Icon>
        )}
      </motion.button>
    );
  },
);
