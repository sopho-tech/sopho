import FormStyles from "./Form.module.css";

type FieldErrorProps = {
  errors: string[];
};

export function FieldError({ errors }: FieldErrorProps) {
  if (errors.length === 0) {
    return null;
  }

  return <div className={FormStyles.formMessage}>{errors[0]}</div>;
}

