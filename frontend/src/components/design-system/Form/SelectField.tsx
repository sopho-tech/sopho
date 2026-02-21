import { useFieldContext } from "src/components/design-system/Form/form-context";
import { Select, SelectOption } from "src/components/design-system";

type SelectFieldProps = {
  groupName: string;
  placeholderText: string;
  options: SelectOption[];
  onChange?: (value: string) => void;
  readonly?: boolean;
};

export function SelectField({
  groupName,
  placeholderText,
  options,
  onChange,
  readonly = false,
}: SelectFieldProps) {
  const field = useFieldContext<string>();

  return (
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
  );
}
