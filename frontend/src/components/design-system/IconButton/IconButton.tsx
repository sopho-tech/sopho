import {
  IconColor,
  IconSize,
  IconType,
} from "src/components/design-system/datatypes";
import styles from "src/components/design-system/IconButton/IconButton.module.css";
import { Icon } from "src/components/design-system/Icon";
import { SophoToolTip } from "src/components/SophoToolTip/SophoToolTip";

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
};

export function IconButton({
  type,
  backgroundColor,
  iconColor,
  iconSize,
  tooltip,
  onClick,
}: IconButtonProps) {
  const className = styles[backgroundColor];
  const button = (
    <button className={`${styles.button} ${className}`} onClick={onClick}>
      <Icon color={iconColor} type={type} size={iconSize}></Icon>
    </button>
  );

  if (tooltip) {
    return (
      <SophoToolTip
        messageElement={tooltip.text}
        tooltipSide={tooltip.direction || "top"}
      >
        {button}
      </SophoToolTip>
    );
  }

  return button;
}
