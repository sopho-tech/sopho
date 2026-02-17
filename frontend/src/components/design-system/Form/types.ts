import { IconType } from "src/components/design-system/datatypes";

export enum FormFieldType {
  INPUT = "INPUT",
  INPUT_PASSWORD = "INPUT_PASSWORD",
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
  validatorFunction: (value: unknown) => string | undefined;
};

export type FormField = {
  key: string;
  name: string | React.ReactNode;
  required?: boolean;
  errorMessage?: string;
  type: FormFieldType;
  initialValue?: string;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  options?: { value: string; label: string | React.ReactNode; textValue?: string }[];
  selectedValue?: string;
  placeholder?: string;
  defaultValue?: string | number | null;
  disabled?: boolean;
  collapsibleConfig?: CollapsibleConfig;
  validator?: FieldValidator;
  infoIconToolTipMessage?: React.ReactNode;
  showLabel?: boolean;
  icon?: IconType;
};

