import { useFieldContext } from "src/components/design-system/Form/form-context";
import { Input } from "src/components/design-system/Input";

type TextFieldProps = {
  label: string;
};

export function TextField({ label }: TextFieldProps) {
  const field = useFieldContext<string>();
  return (
    <label>
      <span>{label}</span>
      <Input
        value={field.state.value}
        onChange={(e) => field.handleChange(e.target.value)}
      />
    </label>
  );
}
