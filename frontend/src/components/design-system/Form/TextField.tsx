import { useFieldContext } from "src/components/design-system/Form/form-context";

type TextFieldProps = {
  label: string;
};

export function TextField({ label }: TextFieldProps) {
  const field = useFieldContext<string>();
  return (
    <label>
      <span>{label}</span>
      <input value={field.state.value} />
    </label>
  );
}
