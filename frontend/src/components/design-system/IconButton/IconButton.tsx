import {
  IconColor,
  IconSize,
  IconType,
  IconButtonSize,
  IconAnimation,
} from "src/components/design-system/datatypes";
import styles from "src/components/design-system/IconButton/IconButton.module.css";
import { Icon } from "src/components/design-system/Icon";
import { ToolTip } from "src/components/design-system/ToolTip";

type IconButtonProps = {
  type: IconType;
  backgroundColor: IconColor;
  iconColor: IconColor;
  size?: IconButtonSize;
  iconSize?: IconSize;
  tooltip?: {
    text?: string;
    content?: React.ReactNode;
    direction?: "top" | "right" | "bottom" | "left";
  };
  onClick?: () => void;
  tabIndex?: number;
  className?: string;
  disabled?: boolean;
  busy?: boolean;
  busyAnimation?: IconAnimation;
  ref?: React.Ref<HTMLButtonElement>;
} & Omit<
  React.ButtonHTMLAttributes<HTMLButtonElement>,
  "type" | "onClick" | "className" | "disabled" | "tabIndex"
>;

export function IconButton({
  type,
  backgroundColor,
  iconColor,
  size = "none",
  iconSize,
  tooltip,
  onClick,
  tabIndex,
  className: customClassName,
  disabled = false,
  busy = false,
  busyAnimation = "pulse",
  ref,
  ...buttonProps
}: IconButtonProps) {
  const backgroundColorClassName = styles[backgroundColor];
  const sizeClassNames: Record<IconButtonSize, string> = {
    none: styles.sizeNone,
    sm: styles.sizeSm,
    md: styles.sizeMd,
    lg: styles.sizeLg,
  };
  const sizeClassName = sizeClassNames[size];
  const button = (
    <button
      ref={ref}
      className={`${styles.button} ${sizeClassName} ${backgroundColorClassName} ${customClassName ?? ""}`.trim()}
      onClick={onClick}
      tabIndex={tabIndex}
      disabled={disabled || busy}
      aria-busy={busy || undefined}
      {...buttonProps}
    >
      <Icon
        color={iconColor}
        type={type}
        size={iconSize}
        animation={busy ? busyAnimation : undefined}
      ></Icon>
    </button>
  );

  if (tooltip) {
    return (
      <ToolTip
        messageElement={tooltip.content ?? tooltip.text}
        tooltipSide={tooltip.direction || "top"}
      >
        {button}
      </ToolTip>
    );
  }

  return button;
}
