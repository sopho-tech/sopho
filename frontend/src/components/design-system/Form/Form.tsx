import { useAppForm } from "src/components/design-system/Form/hooks";
import { FormField } from "src/components/design-system/Form/types";
import { Button } from "src/components/design-system/Button";
import { useState } from "react";
import FormStyles from "src/components/design-system/Form/Form.module.css";
import { SubscribeButton } from "src/components/design-system/Form/SubscribeButton";
import { FormFieldRenderer } from "./FormFieldRenderer";
import {
  validateFields,
  findAccordionsWithInvalidFields,
} from "./utils/validation";
import { deriveDefaultValues, convertValuesToFormData } from "./utils/values";
import { BannerSlim } from "../BannerSlim";
import { revalidateLogic, useStore } from "@tanstack/react-form";
import { getErrorSummary } from "./utils/errorSummary";

type FormProps = {
  fields: FormField[];
  onSubmitCallback: (formData: FormData) => void;
  onCancelCallback: () => void;
  onChange?: (formData: FormData, fieldName: string, value: string) => void;
  submitButtonText?: string;
  showCancelButton?: boolean;
  showSubmitButton?: boolean;
  additionalButtons?: React.ReactNode[];
  defaultValues?: Record<string, any>;
  fieldsContainerStyleClass?: string;
  fieldStyleClass?: string;
  rootStyleClass?: string;
};

export function Form({
  fields,
  onSubmitCallback,
  onCancelCallback,
  onChange,
  submitButtonText = "Submit",
  showCancelButton = true,
  showSubmitButton = true,
  additionalButtons,
  defaultValues,
  fieldsContainerStyleClass,
  fieldStyleClass,
  rootStyleClass,
}: FormProps) {
  const finalDefaultValues = defaultValues || deriveDefaultValues(fields);
  const [accordionState, setAccordionState] = useState<Map<string, boolean>>(
    new Map()
  );

  const form = useAppForm({
    defaultValues: finalDefaultValues,
    onSubmit: ({ value }: { value: unknown }) => {
      const formData = convertValuesToFormData(value as Record<string, any>);
      onSubmitCallback(formData);
    },
    validationLogic: revalidateLogic({
      mode: "submit",
      modeAfterSubmission: "change",
    }),
    validators: {
      onDynamic({ value }) {
        const { errorMap, invalidFields } = validateFields(
          fields,
          value as Record<string, any>
        );

        if (invalidFields.length > 0) {
          const shouldBeOpenAccordions = findAccordionsWithInvalidFields(
            invalidFields,
            fields
          );
          const newAccordionState = new Map(accordionState);
          shouldBeOpenAccordions.forEach((key) => {
            newAccordionState.set(key, true);
          });
          setAccordionState(newAccordionState);

          return {
            form: "form validation",
            fields: errorMap,
          };
        }
        return undefined;
      },
    },
    ...(onChange && {
      listeners: {
        onChange: ({ formApi, fieldApi }) => {
          const fieldName = fieldApi.name;
          const fieldValue = fieldApi.state.value;
          const currentValues = (formApi.state.values || {}) as Record<
            string,
            any
          >;
          const formData = convertValuesToFormData(currentValues);
          onChange(formData, fieldName, String(fieldValue || ""));
        },
      },
    }),
  });

  const formFieldMeta = useStore(form.store, (state) => state.fieldMeta);

  return (
    <div className={`${FormStyles.formRoot} ${rootStyleClass || ""}`}>
      <BannerSlim
        type="error"
        message={getErrorSummary(formFieldMeta, fields)}
      />
      <form.AppForm>
        <div className={fieldsContainerStyleClass || FormStyles.formElements}>
          {fields.map((field) => (
            <FormFieldRenderer
              key={field.key}
              field={field}
              form={form}
              accordionState={accordionState}
              setAccordionState={setAccordionState}
              fieldStyleClass={fieldStyleClass}
            />
          ))}
        </div>
        {(showCancelButton || showSubmitButton || additionalButtons) && (
          <div className={FormStyles.formButtonRow}>
            {showCancelButton && (
              <Button
                label="Cancel"
                onClick={onCancelCallback}
                backgroundColor="white"
                size="sm"
                shape="rectangle"
              />
            )}
            {showSubmitButton && <SubscribeButton label={submitButtonText} />}
            {additionalButtons}
          </div>
        )}
      </form.AppForm>
    </div>
  );
}
