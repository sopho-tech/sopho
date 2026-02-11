import { useFieldContext } from "src/components/design-system/Form/form-context";
import { Select, SelectOption } from "src/components/design-system";
import FormStyles from "src/components/design-system/Form/Form.module.css";
import { Fragment } from "react/jsx-runtime";
import {
  InfoIconTooltip,
  InfoIconTooltipStyles,
} from "src/components/design-system";

type SelectFieldProps = {
  label?: string;
  groupName: string;
  placeholderText: string;
  options: SelectOption[];
  onChange?: (value: string) => void;
  infoIconToolTipMessage?: React.ReactNode;
  showLabel?: boolean;
  readonly?: boolean;
  labelStyleClass?: string;
};

export function SelectField({
  label,
  groupName,
  placeholderText,
  options,
  onChange,
  infoIconToolTipMessage,
  showLabel = true,
  readonly = false,
  labelStyleClass,
}: SelectFieldProps) {
  const field = useFieldContext<string>();

  function renderInfoIconTooltip() {
    if (!infoIconToolTipMessage) {
      return null;
    }
    return (
      <div className={InfoIconTooltipStyles.hoverIcon}>
        <InfoIconTooltip messageElement={infoIconToolTipMessage} />
      </div>
    );
  }

  return (
    <Fragment>
      {((showLabel && label) || infoIconToolTipMessage) && (
        <div
          className={`${FormStyles.formLabelIconContainer} ${InfoIconTooltipStyles.hoverContainer}`}
        >
          {showLabel && label && (
            <span className={labelStyleClass || FormStyles.formLabel}>
              {label}
            </span>
          )}
          {renderInfoIconTooltip()}
        </div>
      )}
      <Select
        value={field.state.value}
        onValueChange={(value) => {
          if (!readonly) {
            field.handleChange(value);
            onChange?.(value);
          }
        }}
        disabled={readonly}
      >
        <Select.Trigger placeholder={placeholderText} />
        <Select.Content>
          <Select.Group>
            <Select.Label>{groupName}</Select.Label>
            {options.map((opt) => (
              <Select.Item
                key={opt.value}
                value={opt.value}
                textValue={opt.textValue}
              >
                {opt.label}
              </Select.Item>
            ))}
          </Select.Group>
        </Select.Content>
      </Select>
    </Fragment>
  );
}
