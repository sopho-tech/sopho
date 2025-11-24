import { IconType } from "src/components/design-system/datatypes";
import styles from "src/components/design-system/IconButtonLink/IconButtonLink.module.css";
import { Icon } from "src/components/design-system/Icon";
import { SophoToolTip } from "src/components/SophoToolTip/SophoToolTip";

export enum State {
  ACTIVE = "ACTIVE",
  DISABLED = "DISABLED",
  INACTIVE = "INACTIVE",
}

type IconButtonLinkProps = {
  state: State;
  type: IconType;
  tooltip?: {
    text: string;
    direction?: "top" | "right" | "bottom" | "left";
  };
};

export function IconButtonLink({ state, type, tooltip }: IconButtonLinkProps) {
  const iconColor = state === State.ACTIVE ? "white" : "black";
  const className = styles[state.toLowerCase()];

  const button = (
    <button className={`${styles.button} ${className}`}>
      <Icon type={type} color={iconColor}></Icon>
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
