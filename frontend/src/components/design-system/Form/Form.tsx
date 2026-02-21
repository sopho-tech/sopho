import { useAppForm } from "src/components/design-system/Form/hooks";
import { Button } from "src/components/design-system/Button";
import { createContext, useContext } from "react";
import FormStyles from "src/components/design-system/Form/Form.module.css";
import { SubscribeButton } from "src/components/design-system/Form/SubscribeButton";
import { TextField } from "./TextField";
import { SelectField } from "./SelectField";
import { PasswordField } from "./PasswordField";
import { FormLabel } from "./FormLabel";
import { convertValuesToFormData } from "./utils/values";
import { BannerSlim } from "../BannerSlim";
import { revalidateLogic, useStore } from "@tanstack/react-form";
import { getErrorSummary } from "./utils/errorSummary";
import { SelectOption } from "src/components/design-system";
import { ButtonSize, IconType } from "src/components/design-system/datatypes";

type FormContextType = {
  form: ReturnType<typeof useAppForm>;
  readonly: boolean;
  formFieldMeta: unknown;
};

const FormCompoundContext = createContext<FormContextType | null>(null);

export const useFormCompoundContext = () => {
  const context = useContext(FormCompoundContext);
  if (!context) {
    throw "Form compound components must be used within a Form parent component";
  }
  return context;
};

type FormRootProps = {
  children: React.ReactNode;
  defaultValues?: Record<string, unknown>;
  onSubmit: (formData: FormData) => void;
  onChange?: (formData: FormData, fieldName: string, value: string) => void;
  className?: string;
  readonly?: boolean;
  submitOnEnter?: boolean;
};

const FormRoot = ({
  children,
  defaultValues = {},
  onSubmit,
  onChange,
  className,
  readonly = false,
  submitOnEnter = false,
}: FormRootProps) => {
  const form = useAppForm({
    defaultValues,
    onSubmit: ({ value }: { value: unknown }) => {
      const formData = convertValuesToFormData(
        value as Record<string, unknown>
      );
      onSubmit(formData);
    },
    validationLogic: revalidateLogic({
      mode: "submit",
      modeAfterSubmission: "change",
    }),
    ...(onChange && {
      listeners: {
        onChange: ({ formApi, fieldApi }) => {
          const fieldName = fieldApi.name;
          const fieldValue = fieldApi.state.value;
          const currentValues = (formApi.state.values || {}) as Record<
            string,
            unknown
          >;
          const formData = convertValuesToFormData(currentValues);
          onChange(formData, fieldName, String(fieldValue || ""));
        },
      },
    }),
  });

  const formFieldMeta = useStore(form.store, (state) => state.fieldMeta);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (submitOnEnter && e.key === "Enter") {
      e.preventDefault();
      form.handleSubmit();
    }
  };

  const contextValue: FormContextType = {
    form,
    readonly,
    formFieldMeta,
  };

  return (
    <FormCompoundContext value={contextValue}>
      <div
        className={`${FormStyles.formRoot} ${className || ""}`}
        onKeyDown={submitOnEnter ? handleKeyDown : undefined}
      >
        <form.AppForm>{children}</form.AppForm>
      </div>
    </FormCompoundContext>
  );
};

FormRoot.displayName = "Form";

const FormErrorBanner = () => {
  const { formFieldMeta } = useFormCompoundContext();
  return <BannerSlim type="error" message={getErrorSummary(formFieldMeta)} />;
};

FormErrorBanner.displayName = "Form.ErrorBanner";

type FormFieldsProps = {
  children: React.ReactNode;
  className?: string;
};

const FormFields = ({ children, className }: FormFieldsProps) => {
  return (
    <div className={`${FormStyles.formElements} ${className || ""}`}>
      {children}
    </div>
  );
};

FormFields.displayName = "Form.Fields";

type FormInputProps = {
  placeholder?: string;
  icon?: IconType;
  className?: string;
  inputContainerClassName?: string;
};

const FormInput = ({
  placeholder,
  icon,
  className,
  inputContainerClassName,
}: FormInputProps) => {
  const { readonly } = useFormCompoundContext();

  return (
    <div className={`${FormStyles.formFieldContainer} ${className || ""}`}>
      <TextField
        placeholder={placeholder}
        icon={icon}
        readonly={readonly}
        containerClassName={inputContainerClassName}
      />
    </div>
  );
};

FormInput.displayName = "Form.Input";

type FormLabelProps = {
  children?: React.ReactNode;
  infoIconToolTipMessage?: React.ReactNode;
  className?: string;
  labelIconContainerStyleClass?: string;
};

const FormLabelComponent = ({
  children,
  infoIconToolTipMessage,
  className,
  labelIconContainerStyleClass,
}: FormLabelProps) => (
  <FormLabel
    infoIconToolTipMessage={infoIconToolTipMessage}
    className={className}
    labelIconContainerStyleClass={labelIconContainerStyleClass}
  >
    {children}
  </FormLabel>
);

FormLabelComponent.displayName = "Form.Label";

type FormFieldProps = {
  name: string;
  required?: boolean;
  errorMessage?: string;
  className?: string;
  children: React.ReactNode;
};

const FormField = ({
  name,
  required = false,
  errorMessage = "Required",
  className,
  children,
}: FormFieldProps) => {
  const { form } = useFormCompoundContext();

  return (
    <form.AppField
      name={name as any}
      validators={
        required
          ? {
              onSubmit: ({ value }: { value: any }) =>
                value === "" || value == null ? errorMessage : undefined,
            }
          : undefined
      }
    >
      {() => (
        <div className={`${FormStyles.formFieldContainer} ${className || ""}`}>
          {children}
        </div>
      )}
    </form.AppField>
  );
};

FormField.displayName = "Form.Field";

type FormSelectProps = {
  groupName?: string;
  placeholder?: string;
  options: SelectOption[];
};

const FormSelect = ({
  groupName = "Options",
  placeholder = "Select an option",
  options,
}: FormSelectProps) => {
  const { readonly } = useFormCompoundContext();

  return (
    <SelectField
      groupName={groupName}
      placeholderText={placeholder}
      options={options}
      readonly={readonly}
    />
  );
};

FormSelect.displayName = "Form.Select";

type FormPasswordProps = {
  placeholder?: string;
  icon?: IconType;
  className?: string;
  inputContainerClassName?: string;
};

const FormPassword = ({
  placeholder,
  icon,
  className,
  inputContainerClassName,
}: FormPasswordProps) => {
  const { readonly } = useFormCompoundContext();

  return (
    <div className={`${FormStyles.formFieldContainer} ${className || ""}`}>
      <PasswordField
        placeholder={placeholder}
        icon={icon}
        readonly={readonly}
        containerClassName={inputContainerClassName}
      />
    </div>
  );
};

FormPassword.displayName = "Form.Password";

type FormActionsProps = {
  children: React.ReactNode;
  className?: string;
};

const FormActions = ({ children, className }: FormActionsProps) => {
  return (
    <div className={`${FormStyles.formButtonRow} ${className || ""}`}>
      {children}
    </div>
  );
};

FormActions.displayName = "Form.Actions";

type FormSubmitProps = {
  label?: string;
  size?: ButtonSize;
};

const FormSubmit = ({ label = "Submit", size = "sm" }: FormSubmitProps) => {
  return <SubscribeButton label={label} size={size} />;
};

FormSubmit.displayName = "Form.Submit";

type FormCancelProps = {
  onClick: () => void;
  label?: string;
};

const FormCancel = ({ onClick, label = "Cancel" }: FormCancelProps) => {
  return (
    <Button
      label={label}
      onClick={onClick}
      backgroundColor="white"
      size="sm"
      shape="rectangle"
    />
  );
};

FormCancel.displayName = "Form.Cancel";

export const Form = Object.assign(FormRoot, {
  ErrorBanner: FormErrorBanner,
  Fields: FormFields,
  Input: FormInput,
  Label: FormLabelComponent,
  Field: FormField,
  Select: FormSelect,
  Password: FormPassword,
  Actions: FormActions,
  Submit: FormSubmit,
  Cancel: FormCancel,
});
