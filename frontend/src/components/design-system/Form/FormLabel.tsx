import { useContext } from "react";
import FormStyles from "src/components/design-system/Form/Form.module.css";
import { FormFieldIdContext } from "src/components/design-system/Form/form-context";
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
  const formFieldId = useContext(FormFieldIdContext);

  if (!children && !infoIconToolTipMessage) {
    return null;
  }

  const labelClassName = `${FormStyles.formLabel} ${className ?? ""}`;

  return (
    <div
      className={`${FormStyles.formLabelIconContainer} ${InfoIconTooltipStyles.hoverContainer} ${labelIconContainerStyleClass ?? ""}`}
    >
      {children &&
        (formFieldId === null ? (
          <span className={labelClassName}>{children}</span>
        ) : (
          <label htmlFor={formFieldId} className={labelClassName}>
            {children}
          </label>
        ))}
      {infoIconToolTipMessage && (
        <div className={InfoIconTooltipStyles.hoverIcon}>
          <InfoIconTooltip messageElement={infoIconToolTipMessage} />
        </div>
      )}
    </div>
  );
}
