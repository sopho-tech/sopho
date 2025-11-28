import { useFieldContext } from "src/components/design-system/Form/form-context";
import { Select, SelectOption } from "src/components/design-system";
import FormStyles from "src/components/design-system/Form/Form.module.css";
import { Fragment } from "react/jsx-runtime";

type SelectFieldProps = {
  label: string;
  groupName: string;
  placeholderText: string;
  options: SelectOption[];
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
    <Fragment>
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
    </Fragment>
  );
}
