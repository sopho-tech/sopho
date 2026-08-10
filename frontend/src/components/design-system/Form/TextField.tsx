import { useFieldContext } from "src/components/design-system/Form/form-context";
import { Input } from "src/components/design-system/Input";
import { IconType } from "src/components/design-system/datatypes";
import { FieldError } from "./FieldError";

type TextFieldProps = {
  placeholder?: string;
  icon?: IconType;
  readonly?: boolean;
  containerClassName?: string;
};

export function TextField({
  placeholder,
  icon,
  readonly = false,
  containerClassName,
}: TextFieldProps) {
  const field = useFieldContext<string>();
  const fieldName = field.name;
  return (
    <>
      <Input
        type="text"
        value={field.state.value ?? ""}
        onChange={(e) => field.handleChange(e.target.value)}
        placeholder={placeholder}
        leadingIcon={icon}
        id={fieldName}
        name={fieldName}
        disabled={readonly}
        containerClassName={containerClassName}
      />
      <FieldError errors={field.state.meta.errors} />
    </>
  );
}
