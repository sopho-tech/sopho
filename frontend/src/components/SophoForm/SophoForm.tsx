import * as Form from "@radix-ui/react-form";
import FormStyles from "src/components/SophoForm/SophoForm.module.css";
import { ButtonStyle, NewAssetButton } from "src/components/NewAssetButton";
import { useMemo } from "react";
import { mergeForms, getInitialFormValues } from "src/utils/form_utils";

export type SophoFormElement = {
  key: string;
  name: string;
  required: boolean;
  error_message: string;
  type: SophoFormElementType;
  initialValue?: string;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  options?: { value: string; label: string }[];
  selectedValue?: string;
  placeholder?: string;
  input_type?: InputType;
  defaultValue?: string | number | null;
  disabled?: boolean;
};

export type SophoFormProps = {
  initialFormData?: FormData;
  formElements: SophoFormElement[];
  onSubmitCallback: (formData: FormData) => void;
  onCancelCallback: any;
  submitButtonText?: string;
  showCancelButton?: boolean;
  showSubmitButton?: boolean;
  formElementsStyleClass?: string;
};

export enum SophoFormElementType {
  INPUT = "INPUT",
  SELECT = "SELECT",
}

export enum InputType {
  TEXT = "text",
  EMAIL = "email",
  PASSWORD = "password",
  NUMBER = "number",
}

function renderFormControl(formElement: SophoFormElement) {
  if (formElement.type === SophoFormElementType.SELECT) {
    return (
      <select
        className={FormStyles.formSelect}
        required={formElement.required}
        name={formElement.key}
        defaultValue={
          formElement.selectedValue || formElement.defaultValue || ""
        }
        disabled={formElement.disabled}
      >
        <option value="">Select an option</option>
        {formElement.options?.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    );
  }

  return (
    <input
      className={FormStyles.formInput}
      required={formElement.required}
      name={formElement.key}
      type={formElement.input_type || InputType.TEXT}
      placeholder={formElement.placeholder}
      defaultValue={formElement.defaultValue ?? formElement.initialValue ?? ""}
      onChange={formElement.onChange}
      disabled={formElement.disabled}
    />
  );
}

export function SophoForm({
  initialFormData = new FormData(),
  formElements,
  onSubmitCallback,
  onCancelCallback,
  submitButtonText = "Submit",
  showCancelButton = true,
  showSubmitButton = true,
  formElementsStyleClass,
}: SophoFormProps) {
  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const finalFormData = mergeForms(initialFormData, event.currentTarget);
    console.log("formData submitted:", Object.fromEntries(finalFormData));
    onSubmitCallback(finalFormData);
  }

  return (
    <Form.Root className={FormStyles.formRoot} onSubmit={handleSubmit}>
      <div className={formElementsStyleClass || FormStyles.formElements}>
        {formElements.map((formElement) => {
          return (
            <Form.Field
              className={FormStyles.formField}
              name={formElement.key}
              key={formElement.key}
            >
              <Form.Label className={FormStyles.formLabel}>
                {formElement.name}
              </Form.Label>
              <Form.Control asChild>
                {renderFormControl(formElement)}
              </Form.Control>
              <Form.Message
                className={FormStyles.formMessage}
                match="valueMissing"
              >
                {formElement.error_message}
              </Form.Message>
            </Form.Field>
          );
        })}
      </div>
      {(showCancelButton || showSubmitButton) && (
        <div className={FormStyles.formButtonRow}>
          {showCancelButton && (
            <NewAssetButton
              buttonText="Cancel"
              isLoading={false}
              onClick={onCancelCallback}
              style={ButtonStyle.BackButton}
            />
          )}
          {showSubmitButton && (
            <Form.Submit asChild>
              <NewAssetButton
                buttonText={submitButtonText}
                isLoading={false}
                onClick={() => {}}
              />
            </Form.Submit>
          )}
        </div>
      )}
    </Form.Root>
  );
}
