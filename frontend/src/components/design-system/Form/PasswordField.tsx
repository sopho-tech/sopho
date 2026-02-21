import { useState } from "react";
import { useFieldContext } from "src/components/design-system/Form/form-context";
import { IconButton } from "src/components/design-system/IconButton";
import { Input } from "src/components/design-system/Input";
import { IconType } from "src/components/design-system/datatypes";
import styles from "./PasswordField.module.css";
import { FieldError } from "./FieldError";

type PasswordFieldProps = {
  placeholder?: string;
  icon?: IconType;
  readonly?: boolean;
  containerClassName?: string;
};

export function PasswordField({
  placeholder,
  icon,
  readonly = false,
  containerClassName,
}: PasswordFieldProps) {
  const field = useFieldContext<string>();
  const [showPassword, setShowPassword] = useState(false);
  const fieldName = field.name;

  const togglePasswordVisibility = () => {
    if (!readonly) {
      setShowPassword((prev) => !prev);
    }
  };

  return (
    <div className={`${styles.passwordContainer} ${containerClassName ?? ""}`.trim()}>
        <div className={styles.passwordInputWrapper}>
          <Input
            type={showPassword ? "text" : "password"}
            value={field.state.value}
            onChange={(e) => field.handleChange(e.target.value)}
            placeholder={placeholder}
            leadingIcon={icon}
            id={fieldName}
            name={fieldName}
            disabled={readonly}
          />
          {!readonly && (
            <div className={styles.visibilityButton}>
              <IconButton
                type={showPassword ? "visibility_off" : "visibility"}
                backgroundColor="transparent"
                iconColor="grey"
                iconSize="sm"
                onClick={togglePasswordVisibility}
                tabIndex={-1}
              />
            </div>
          )}
        </div>

        <FieldError errors={field.state.meta.errors} />
    </div>
  );
}
