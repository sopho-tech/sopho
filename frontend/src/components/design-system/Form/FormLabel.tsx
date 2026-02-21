import FormStyles from "src/components/design-system/Form/Form.module.css";
import {
  InfoIconTooltip,
  InfoIconTooltipStyles,
} from "src/components/design-system";

type FormLabelProps = {
  children?: React.ReactNode;
  infoIconToolTipMessage?: React.ReactNode;
  className?: string;
  labelIconContainerStyleClass?: string;
};

export function FormLabel({
  children,
  infoIconToolTipMessage,
  className,
  labelIconContainerStyleClass,
}: FormLabelProps) {
  if (!children && !infoIconToolTipMessage) {
    return null;
  }

  return (
    <div
      className={`${FormStyles.formLabelIconContainer} ${InfoIconTooltipStyles.hoverContainer} ${labelIconContainerStyleClass ?? ""}`}
    >
      {children && (
        <span className={`${FormStyles.formLabel} ${className ?? ""}`}>
          {children}
        </span>
      )}
      {infoIconToolTipMessage && (
        <div className={InfoIconTooltipStyles.hoverIcon}>
          <InfoIconTooltip messageElement={infoIconToolTipMessage} />
        </div>
      )}
    </div>
  );
}
