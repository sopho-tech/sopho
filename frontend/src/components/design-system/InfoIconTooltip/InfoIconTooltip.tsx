import { ToolTip } from "src/components/design-system/ToolTip";
import { Icon } from "src/components/design-system";
import styles from "src/components/design-system/InfoIconTooltip/InfoIconTooltip.module.css";

type InfoIconTooltipProps = {
  messageElement: React.ReactNode;
};

export function InfoIconTooltip({ messageElement }: InfoIconTooltipProps) {
  const toolTipTrigger = (
    <span className={styles.container}>
      <Icon type="info" color="grey" size="sm" />
    </span>
  );
  return (
    <ToolTip messageElement={messageElement} children={toolTipTrigger} />
  );
}
