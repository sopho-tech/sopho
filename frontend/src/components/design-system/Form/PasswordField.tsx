import { useState } from "react";
import { useFieldContext } from "src/components/design-system/Form/form-context";
import { IconButton } from "src/components/design-system/IconButton";
import { Input } from "src/components/design-system/Input";
import { IconType } from "src/components/design-system/datatypes";
import styles from "./PasswordField.module.css";
import { FieldError } from "./FieldError";

type PasswordFieldProps = {
  label?: string;
  placeholder?: string;
  showLabel?: boolean;
  icon?: IconType;
  readonly?: boolean;
  containerStyleClass?: string;
  labelStyleClass?: string;
};

export function PasswordField({
  label,
  placeholder,
  showLabel = true,
  icon,
  readonly = false,
  containerStyleClass,
  labelStyleClass,
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
    <label className={containerStyleClass}>
      {showLabel && label && <span className={labelStyleClass}>{label}</span>}
      <div className={styles.passwordContainer}>
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
    </label>
  );
}
