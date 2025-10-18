import * as Form from "@radix-ui/react-form";
import { useState } from "react";
import FormStyles from "src/components/SophoForm/SophoForm.module.css";
import {
  SophoFormElement,
  SophoFormElementType,
  InputType,
} from "src/components/SophoForm";
import { mergeForms } from "src/utils/form_utils";
import { ButtonStyle, NewAssetButton } from "src/components/NewAssetButton";

export enum SophoMultiStepFormLayout {
  VERTICAL = "VERTICAL",
  HORIZONTAL = "HORIZONTAL",
}

export type SophoMultiStepFormStep = {
  formElements: SophoFormElement[];
  backButtonText?: string;
  nextButtonText?: string;
  showBackButton?: boolean;
  showNextButton?: boolean;
};

export type SophoMultiStepFormProps = {
  steps: SophoMultiStepFormStep[];
  type: SophoMultiStepFormLayout;
  onSubmitCallback: (formData: FormData) => void;
  onProgressChange: (newProgressValue: number) => void;
  formElementsStyleClass?: string;
  buttomRowStyleClass?: string;
};

function renderFormControl(formElement: SophoFormElement, formData: FormData) {
  if (formElement.type === SophoFormElementType.SELECT) {
    const defaultValue =
      (formData.get(formElement.key) as string) ||
      formElement.selectedValue ||
      "";
    return (
      <select
        className={FormStyles.formSelect}
        required={formElement.required}
        name={formElement.key}
        defaultValue={defaultValue}
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
      type={formElement.input_type || InputType.TEXT}
      className={FormStyles.formInput}
      required={formElement.required}
      name={formElement.key}
      value={formElement.initialValue}
      onChange={formElement.onChange}
      placeholder={formElement.placeholder}
    />
  );
}

function renderButtonRow(
  currentStep: SophoMultiStepFormStep,
  handleBackCallback: () => void,
  isFirstStep: boolean,
  isLastStep: boolean,
  buttomRowStyleClass: string
) {
  const showBack = currentStep.showBackButton !== false && !isFirstStep;
  const showNext = currentStep.showNextButton !== false;

  if (!showBack && !showNext) {
    return null;
  }

  const defaultNextText = isLastStep ? "Submit" : "Next";
  const nextButtonText = currentStep.nextButtonText ?? defaultNextText;

  return (
    <div className={`${FormStyles.formButtonRow} ${buttomRowStyleClass || ""}`}>
      {showBack && (
        <NewAssetButton
          style={ButtonStyle.BackButton}
          buttonText={currentStep.backButtonText ?? "Back"}
          isLoading={false}
          onClick={handleBackCallback}
        />
      )}
      {showNext && (
        <Form.Submit asChild>
          <NewAssetButton
            buttonText={nextButtonText}
            isLoading={false}
            onClick={() => {}}
          />
        </Form.Submit>
      )}
    </div>
  );
}

export function SophoMultiStepForm({
  steps,
  type,
  onSubmitCallback,
  onProgressChange,
  formElementsStyleClass,
  buttomRowStyleClass,
}: SophoMultiStepFormProps) {
  // TODO: Throw Error if a key is repeated across the steps
  const [formData, setFormData] = useState<FormData>(new FormData());
  const [stepNumber, setStepNumber] = useState<number>(1);
  const totalSteps = steps.length;
  const currentStep = getCurrentStep();
  const formElements = currentStep.formElements;
  const isFirstStep = stepNumber === 1;
  const isLastStep = stepNumber === totalSteps;

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const finalFormData = mergeForms(formData, event.currentTarget);
    setFormData(finalFormData);

    if (steps.length === stepNumber) {
      changeStepNumberChange(stepNumber + 1);
      setTimeout(() => {
        onSubmitCallback(finalFormData);
      }, 600);
    } else {
      changeStepNumberChange(stepNumber + 1);
    }
  }

  function handleBackCallback() {
    changeStepNumberChange(stepNumber - 1);
  }

  function changeStepNumberChange(newStepNumber: number) {
    setStepNumber(Math.min(newStepNumber, totalSteps));
    onProgressChange(newStepNumber);
  }

  if (type === SophoMultiStepFormLayout.HORIZONTAL) {
    throw new Error("Horizontal layout is not supported");
  }

  function getCurrentStep() {
    return steps[stepNumber - 1];
  }

  return (
    <Form.Root className={FormStyles.formRoot} onSubmit={handleSubmit}>
      <div
        className={`${FormStyles.formElements} ${formElementsStyleClass || ""}`}
      >
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
                {renderFormControl(formElement, formData)}
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
      {renderButtonRow(
        currentStep,
        handleBackCallback,
        isFirstStep,
        isLastStep,
        buttomRowStyleClass
      )}
    </Form.Root>
  );
}
