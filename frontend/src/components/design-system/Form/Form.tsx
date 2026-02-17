import { useAppForm } from "src/components/design-system/Form/hooks";
import { Button } from "src/components/design-system/Button";
import { createContext, useContext } from "react";
import FormStyles from "src/components/design-system/Form/Form.module.css";
import { SubscribeButton } from "src/components/design-system/Form/SubscribeButton";
import { TextField } from "./TextField";
import { SelectField } from "./SelectField";
import { PasswordField } from "./PasswordField";
import { convertValuesToFormData } from "./utils/values";
import { BannerSlim } from "../BannerSlim";
import { revalidateLogic, useStore } from "@tanstack/react-form";
import { getErrorSummary } from "./utils/errorSummary";
import { SelectOption } from "src/components/design-system";
import { IconType } from "src/components/design-system/datatypes";

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
      const formData = convertValuesToFormData(value as Record<string, unknown>);
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
  name: string;
  label?: string;
  placeholder?: string;
  required?: boolean;
  errorMessage?: string;
  showLabel?: boolean;
  icon?: IconType;
  className?: string;
  labelClassName?: string;
  inputContainerClassName?: string;
};

const FormInput = ({
  name,
  label,
  placeholder,
  required = false,
  errorMessage = "Required",
  showLabel = true,
  icon,
  className,
  labelClassName,
  inputContainerClassName,
}: FormInputProps) => {
  const { form, readonly } = useFormCompoundContext();

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
        // <div className={FormStyles.formFieldContainer}>
        <TextField
          label={label}
          placeholder={placeholder}
          showLabel={showLabel}
          icon={icon}
          readonly={readonly}
          containerStyleClass={className}
          labelStyleClass={labelClassName}
          inputContainerClassName={inputContainerClassName}
        />
        // </div>
      )}
    </form.AppField>
  );
};

FormInput.displayName = "Form.Input";

type FormSelectProps = {
  name: string;
  label?: string;
  groupName?: string;
  placeholder?: string;
  options: SelectOption[];
  required?: boolean;
  errorMessage?: string;
  showLabel?: boolean;
  infoIconToolTipMessage?: React.ReactNode;
  className?: string;
  labelClassName?: string;
  labelIconContainerStyleClass?: string;
};

const FormSelect = ({
  name,
  label,
  groupName,
  placeholder = "Select an option",
  options,
  required = false,
  errorMessage = "Required",
  showLabel = true,
  infoIconToolTipMessage,
  className,
  labelClassName,
  labelIconContainerStyleClass,
}: FormSelectProps) => {
  const { form, readonly } = useFormCompoundContext();

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
          <SelectField
            label={label}
            groupName={groupName || label || "Options"}
            placeholderText={placeholder}
            options={options}
            infoIconToolTipMessage={infoIconToolTipMessage}
            showLabel={showLabel}
            readonly={readonly}
            labelStyleClass={labelClassName}
            labelIconContainerStyleClass={labelIconContainerStyleClass}
          />
        </div>
      )}
    </form.AppField>
  );
};

FormSelect.displayName = "Form.Select";

type FormPasswordProps = {
  name: string;
  label?: string;
  placeholder?: string;
  required?: boolean;
  errorMessage?: string;
  showLabel?: boolean;
  icon?: IconType;
  className?: string;
  labelClassName?: string;
};

const FormPassword = ({
  name,
  label,
  placeholder,
  required = false,
  errorMessage = "Required",
  showLabel = true,
  icon,
  className,
  labelClassName,
}: FormPasswordProps) => {
  const { form, readonly } = useFormCompoundContext();

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
          <PasswordField
            label={label}
            placeholder={placeholder}
            showLabel={showLabel}
            icon={icon}
            readonly={readonly}
            containerStyleClass={className}
            labelStyleClass={labelClassName}
          />
        </div>
      )}
    </form.AppField>
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
};

const FormSubmit = ({ label = "Submit" }: FormSubmitProps) => {
  return <SubscribeButton label={label} />;
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
  Select: FormSelect,
  Password: FormPassword,
  Actions: FormActions,
  Submit: FormSubmit,
  Cancel: FormCancel,
});
