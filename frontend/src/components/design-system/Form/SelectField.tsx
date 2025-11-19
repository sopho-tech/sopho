import { useFieldContext } from "src/components/design-system/Form/form-context";
import { Select } from "src/components/design-system";
import FormStyles from "src/components/design-system/Form/Form.module.css";

type SelectFieldProps = {
  label: string;
  groupName: string;
  placeholderText: string;
  options: { label: string; value: string }[];
  onChange?: (value: string) => void;
};

export function SelectField({
  label,
  groupName,
  placeholderText,
  options,
  onChange,
}: SelectFieldProps) {
  const field = useFieldContext<string>();

  return (
    <label className={FormStyles.formField}>
      <span className={FormStyles.formLabel}>{label}</span>
      <Select
        groupName={groupName}
        onValueChange={(value) => {
          field.handleChange(value);
          onChange?.(value);
        }}
        options={options}
        placeholderText={placeholderText}
        value={field.state.value}
      />
    </label>
  );
}
