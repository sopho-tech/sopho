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
  iconMirrored?: boolean;
  tooltip?: {
    text?: string;
    content?: React.ReactNode;
    direction?: "top" | "right" | "bottom" | "left";
  };
  onClick?: () => void;
  tabIndex?: number;
  className?: string;
  disabled?: boolean;
  disabledReason?: string;
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
  iconMirrored,
  tooltip,
  onClick,
  tabIndex,
  className: customClassName,
  disabled = false,
  disabledReason,
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
  const isDisabledWithReason = disabledReason !== undefined;
  const button = (
    <button
      ref={ref}
      className={`${styles.button} ${sizeClassName} ${backgroundColorClassName} ${customClassName ?? ""}`.trim()}
      onClick={isDisabledWithReason ? undefined : onClick}
      tabIndex={tabIndex}
      disabled={!isDisabledWithReason && (disabled || busy)}
      aria-disabled={isDisabledWithReason || undefined}
      aria-busy={busy || undefined}
      {...buttonProps}
    >
      <Icon
        color={iconColor}
        type={type}
        size={iconSize}
        mirrored={iconMirrored}
        animation={busy ? busyAnimation : undefined}
      ></Icon>
    </button>
  );

  if (tooltip || isDisabledWithReason) {
    return (
      <ToolTip
        messageElement={disabledReason ?? tooltip?.content ?? tooltip?.text}
        tooltipSide={tooltip?.direction || "top"}
      >
        {button}
      </ToolTip>
    );
  }

  return button;
}
