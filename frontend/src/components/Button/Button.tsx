import React from "react";
import ButtonStyles from "./Button.module.css";

export type ButtonVariant = "neutral" | "icon" | "ghost";

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  children: React.ReactNode;
  className?: string;
}

export function Button({
  variant = "neutral",
  disabled = false,
  children,
  className = "",
  ...props
}: ButtonProps) {
  const getVariantClass = () => {
    switch (variant) {
      case "icon":
        return ButtonStyles.icon;
      case "ghost":
        return ButtonStyles.ghost;
      case "neutral":
      default:
        return ButtonStyles.neutral;
    }
  };

  return (
    <button
      className={`${ButtonStyles.button} ${getVariantClass()} ${className}`.trim()}
      disabled={disabled}
      {...props}
    >
      {children}
    </button>
  );
}
