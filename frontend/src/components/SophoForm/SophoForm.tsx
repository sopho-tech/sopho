import * as Form from "@radix-ui/react-form";
import { useState, Fragment, useEffect } from "react";
import FormStyles from "src/components/SophoForm/SophoForm.module.css";
import { Button } from "src/components/design-system/Button";
import { mergeForms } from "src/utils/form_utils";
import { Accordion } from "src/components/design-system/Accordion";

export type CollapsibleConfig = {
  formElements: SophoFormElement[];
};

export type SophoFormElement = {
  key: string;
  name: string | React.ReactNode;
  required?: boolean;
  error_message?: string;
  type: SophoFormElementType;
  initialValue?: string;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  options?: { value: string; label: string }[];
  selectedValue?: string;
  placeholder?: string;
  input_type?: InputType;
  defaultValue?: string | number | null;
  disabled?: boolean;
  collapsibleConfig?: CollapsibleConfig;
};

export type SophoFormProps = {
  initialFormData?: FormData;
  formElements: SophoFormElement[];
  onSubmitCallback: (formData: FormData) => void;
  onCancelCallback: any;
  onChange?: (formData: FormData, fieldName: string, value: string) => void;
  submitButtonText?: string;
  showCancelButton?: boolean;
  showSubmitButton?: boolean;
  additionalButtons?: React.ReactNode[];
  formElementsStyleClass?: string;
  formFieldStyleClass?: string;
  formRootStyleClass?: string;
};

export enum SophoFormElementType {
  INPUT = "INPUT",
  SELECT = "SELECT",
  COLLAPSIBLE = "COLLAPSIBLE",
}

export enum InputType {
  TEXT = "text",
  EMAIL = "email",
  PASSWORD = "password",
  NUMBER = "number",
}

function renderCollapsibleSection(
  collapsibleFormElement: SophoFormElement,
  formFieldStyleClass?: string,
  onFormChange?: (
    event: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => void
): React.ReactNode {
  return (
    <Accordion value={[]} onValueChange={() => {}}>
      <Accordion.Item value={collapsibleFormElement.key}>
        <Accordion.Trigger>{collapsibleFormElement.name}</Accordion.Trigger>
        <Accordion.Content>
          {collapsibleFormElement.collapsibleConfig?.formElements.map(
            (formElement) =>
              renderFormElement(formElement, formFieldStyleClass, onFormChange)
          )}
        </Accordion.Content>
      </Accordion.Item>
    </Accordion>
  );
}

function ControlledSelect({
  formElement,
  onFormChange,
}: {
  formElement: SophoFormElement;
  onFormChange?: (event: React.ChangeEvent<HTMLSelectElement>) => void;
}) {
  const initialValue =
    formElement.selectedValue ||
    (formElement.defaultValue !== null && formElement.defaultValue !== undefined
      ? String(formElement.defaultValue)
      : "") ||
    "";
  const [value, setValue] = useState<string>(initialValue);

  useEffect(() => {
    if (formElement.selectedValue !== undefined) {
      setValue(formElement.selectedValue);
    }
  }, [formElement.selectedValue]);

  return (
    <select
      className={FormStyles.formSelect}
      required={formElement.required}
      name={formElement.key}
      value={value}
      onChange={(e) => {
        setValue(e.target.value);
        onFormChange?.(e);
      }}
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

function renderSelect(
  formElement: SophoFormElement,
  onFormChange?: (event: React.ChangeEvent<HTMLSelectElement>) => void
) {
  if (formElement.selectedValue !== undefined) {
    return (
      <ControlledSelect formElement={formElement} onFormChange={onFormChange} />
    );
  }
  return (
    <select
      className={FormStyles.formSelect}
      required={formElement.required}
      name={formElement.key}
      defaultValue={formElement.defaultValue || ""}
      onChange={onFormChange}
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

function renderFormControl(
  formElement: SophoFormElement,
  onFormChange?: (
    event: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => void
): React.ReactNode {
  switch (formElement.type) {
    case SophoFormElementType.SELECT: {
      return renderSelect(formElement, onFormChange);
    }
    case SophoFormElementType.INPUT: {
      return (
        <input
          className={FormStyles.formInput}
          required={formElement.required}
          name={formElement.key}
          type={formElement.input_type || InputType.TEXT}
          placeholder={formElement.placeholder}
          defaultValue={
            formElement.defaultValue ?? formElement.initialValue ?? ""
          }
          onChange={(e) => {
            formElement.onChange?.(e);
            onFormChange?.(e);
          }}
          disabled={formElement.disabled}
        />
      );
    }
  }
  throw Error("Form element cannot be handled");
}

function renderFormElement(
  formElement: SophoFormElement,
  formFieldStyleClass?: string,
  onFormChange?: (
    event: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => void
): React.ReactNode {
  switch (formElement.type) {
    case SophoFormElementType.INPUT:
    case SophoFormElementType.SELECT: {
      return (
        <Form.Field
          className={`${FormStyles.formField} ${formFieldStyleClass}`}
          name={formElement.key}
          key={formElement.key}
        >
          <div className={FormStyles.formLabelControlRow}>
            <Form.Label className={FormStyles.formLabel}>
              {formElement.name}
            </Form.Label>
            <Form.Control asChild>
              {renderFormControl(formElement, onFormChange)}
            </Form.Control>
          </div>
          <Form.Message
            name={formElement.key}
            className={FormStyles.formMessage}
            match="valueMissing"
          >
            {formElement.error_message}
          </Form.Message>
        </Form.Field>
      );
    }
    case SophoFormElementType.COLLAPSIBLE: {
      return renderCollapsibleSection(
        formElement,
        formFieldStyleClass,
        onFormChange
      );
    }
  }
}

export function SophoForm({
  initialFormData = new FormData(),
  formElements,
  onSubmitCallback,
  onCancelCallback,
  onChange,
  submitButtonText = "Submit",
  showCancelButton = true,
  showSubmitButton = true,
  additionalButtons,
  formElementsStyleClass,
  formFieldStyleClass,
  formRootStyleClass,
}: SophoFormProps) {
  const handleFormChange = (
    event: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    if (onChange) {
      const form = event.currentTarget.form;
      if (form) {
        const formData = new FormData(form);
        onChange(formData, event.target.name, event.target.value);
      }
    }
  };

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const finalFormData = mergeForms(initialFormData, event.currentTarget);
    onSubmitCallback(finalFormData);
  }

  return (
    <Form.Root
      className={`${FormStyles.formRoot} ${formRootStyleClass}`}
      onSubmit={handleSubmit}
    >
      <div className={formElementsStyleClass || FormStyles.formElements}>
        {formElements.map((formElement) => (
          <Fragment key={formElement.key}>
            {renderFormElement(
              formElement,
              formFieldStyleClass,
              handleFormChange
            )}
          </Fragment>
        ))}
      </div>
      {(showCancelButton || showSubmitButton) && (
        <div className={FormStyles.formButtonRow}>
          {showCancelButton && (
            <Button
              label="Cancel"
              onClick={() => onCancelCallback()}
              backgroundColor="white"
              size="sm"
              shape="rectangle"
            />
          )}
          {showSubmitButton && (
            <Form.Submit asChild>
              <Button
                label={submitButtonText}
                backgroundColor="accent"
                size="sm"
                shape="rectangle"
                type="submit"
              />
            </Form.Submit>
          )}
          {additionalButtons}
        </div>
      )}
    </Form.Root>
  );
}
