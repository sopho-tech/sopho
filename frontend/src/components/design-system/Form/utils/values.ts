import { FormField, FormFieldType } from "../types";

/**
 * Derives default values from form fields recursively
 */
export function deriveDefaultValues(fields: FormField[]): Record<string, any> {
  const defaultValues: Record<string, any> = {};

  function processField(field: FormField) {
    const value =
      field.selectedValue ??
      (field.defaultValue != null ? String(field.defaultValue) : field.initialValue ?? "");

    if (value) {
      defaultValues[field.key] = value;
    }

    if (field.type === FormFieldType.COLLAPSIBLE) {
      field.collapsibleConfig?.fields.forEach(processField);
    }
  }

  fields.forEach(processField);
  return defaultValues;
}

/**
 * Converts form values to FormData object
 */
export function convertValuesToFormData(values: Record<string, any>): FormData {
  const formData = new FormData();
  Object.entries(values).forEach(([key, value]) => {
    if (value != null) {
      formData.append(key, String(value));
    }
  });
  return formData;
}

