import { useFieldContext } from "src/components/design-system/Form/form-context";
import { Input } from "src/components/design-system/Input";
import { IconType } from "src/components/design-system/datatypes";
import { FieldError } from "./FieldError";

type TextFieldProps = {
  label?: string;
  placeholder?: string;
  showLabel?: boolean;
  icon?: IconType;
  readonly?: boolean;
  containerStyleClass?: string;
  labelStyleClass?: string;
};

export function TextField({
  label,
  placeholder,
  showLabel = true,
  icon,
  readonly = false,
  containerStyleClass,
  labelStyleClass,
}: TextFieldProps) {
  const field = useFieldContext<string>();
  const fieldName = field.name;
  return (
    <label className={containerStyleClass}>
      {showLabel && label && <span className={labelStyleClass}>{label}</span>}
      <Input
        type="text"
        value={field.state.value}
        onChange={(e) => field.handleChange(e.target.value)}
        placeholder={placeholder}
        leadingIcon={icon}
        id={fieldName}
        name={fieldName}
        disabled={readonly}
      />
      <FieldError errors={field.state.meta.errors} />
    </label>
  );
}
