import {
  IconColor,
  IconSize,
  IconType,
} from "src/components/design-system/datatypes";
import styles from "src/components/design-system/IconButton/IconButton.module.css";
import { Icon } from "src/components/design-system/Icon";
import { ToolTip } from "src/components/design-system/ToolTip";

type IconButtonProps = {
  type: IconType;
  backgroundColor: IconColor;
  iconColor: IconColor;
  iconSize?: IconSize;
  tooltip?: {
    text: string;
    direction?: "top" | "right" | "bottom" | "left";
  };
  onClick: () => void;
  tabIndex?: number;
};

export function IconButton({
  type,
  backgroundColor,
  iconColor,
  iconSize,
  tooltip,
  onClick,
  tabIndex,
}: IconButtonProps) {
  const className = styles[backgroundColor];
  const button = (
    <button
      className={`${styles.button} ${className}`}
      onClick={onClick}
      tabIndex={tabIndex}
    >
      <Icon color={iconColor} type={type} size={iconSize}></Icon>
    </button>
  );

  if (tooltip) {
    return (
      <ToolTip
        messageElement={tooltip.text}
        tooltipSide={tooltip.direction || "top"}
      >
        {button}
      </ToolTip>
    );
  }

  return button;
}
