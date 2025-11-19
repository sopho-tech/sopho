export enum FormFieldType {
  INPUT = "INPUT",
  SELECT = "SELECT",
  COLLAPSIBLE = "COLLAPSIBLE",
}

export enum ValidationTrigger {
  ON_CHANGE = "ON_CHANGE",
  ON_BLUR = "ON_BLUR",
}

export type CollapsibleConfig = {
  fields: FormField[];
};

export type FieldValidator = {
  trigger: ValidationTrigger;
  validatorFunction: (value: any) => string | undefined;
};

export type FormField = {
  key: string;
  name: string | React.ReactNode;
  required?: boolean;
  errorMessage?: string;
  type: FormFieldType;
  initialValue?: string;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  options?: { value: string; label: string }[];
  selectedValue?: string;
  placeholder?: string;
  defaultValue?: string | number | null;
  disabled?: boolean;
  collapsibleConfig?: CollapsibleConfig;
  validator?: FieldValidator;
};

