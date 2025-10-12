import * as Form from "@radix-ui/react-form";
import FormStyles from "src/components/SophoForm/SophoForm.module.css";
import { NewAssetButton } from "src/components/NewAssetButton";
import { useState } from "react";

interface SophoFormProps {
  formElements: SophoFormElement[];
  onSubmitCallback: (formData: FormData) => void;
  onCancelCallback: any;
  submitButtonText?: string;
  showCancelButton?: boolean;
  showSubmitButton?: boolean;
}

export enum SophoFormElementType {
  INPUT = "INPUT",
  SELECT = "SELECT",
}

export type SophoFormElement = {
  key: string;
  name: string;
  required: boolean;
  message: string;
  type: SophoFormElementType;
  initialValue?: string;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  options?: { value: string; label: string }[];
  selectedValue?: string;
};

export function SophoForm({
  formElements,
  onSubmitCallback,
  onCancelCallback,
  submitButtonText = "Submit",
  showCancelButton = true,
  showSubmitButton = true,
}: SophoFormProps) {
  const [submissionError, setSubmissionError] = useState<string | null>(null);

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSubmissionError(null);
    const formData = new FormData(event.currentTarget);
    try {
      await onSubmitCallback(formData);
    } catch (error: any) {
      setSubmissionError(error.message || "An unexpected error occurred.");
    }
  };

  const renderFormControl = (formElement: SophoFormElement) => {
    if (formElement.type === SophoFormElementType.SELECT) {
      return (
        <select
          className={FormStyles.formSelect}
          required={formElement.required}
          name={formElement.key}
          value={formElement.selectedValue || ""}
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
        value={formElement.initialValue}
        onChange={formElement.onChange}
      />
    );
  };

  return (
    <Form.Root className={FormStyles.formRoot} onSubmit={handleSubmit}>
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
              {formElement.message}
            </Form.Message>
          </Form.Field>
        );
      })}
      {submissionError && (
        <div className={FormStyles.formError}>{submissionError}</div>
      )}
      {(showCancelButton || showSubmitButton) && (
        <div className={FormStyles.formButtonRow}>
          {showCancelButton && (
            <NewAssetButton
              buttonText="Cancel"
              isLoading={false}
              onClick={onCancelCallback}
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
