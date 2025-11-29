import { SophoToolTip } from "src/components/SophoToolTip";
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
    <SophoToolTip messageElement={messageElement} children={toolTipTrigger} />
  );
}
