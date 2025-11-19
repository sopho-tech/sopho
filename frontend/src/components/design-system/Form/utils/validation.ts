import { FormField, FormFieldType } from "../types";

export type ValidationResult = {
  errorMap: Record<string, string>;
  invalidFields: string[];
};

/**
 * Checks if a field value is empty (null, undefined, or empty string)
 */
function isEmptyValue(value: any): boolean {
  return value === "" || value === undefined || value === null;
}

/**
 * Validates a single field and adds error to errorMap if invalid
 */
function validateField(
  field: FormField,
  value: any,
  errorMap: Record<string, string>
): void {
  if (field.required && isEmptyValue(value)) {
    errorMap[field.key] = field.errorMessage || "Required";
  }
}

/**
 * Recursively validates all fields in the form
 */
function validateFieldsRecursive(
  fields: FormField[],
  values: Record<string, any>,
  errorMap: Record<string, string>
): void {
  for (const field of fields) {
    if (field.type === FormFieldType.COLLAPSIBLE) {
      if (field.collapsibleConfig) {
        validateFieldsRecursive(
          field.collapsibleConfig.fields,
          values,
          errorMap
        );
      }
    } else {
      const fieldValue = values[field.key];
      validateField(field, fieldValue, errorMap);
    }
  }
}

/**
 * Validates all form fields and returns error map and invalid field keys
 */
export function validateFields(
  fields: FormField[],
  values: Record<string, any>
): ValidationResult {
  const errorMap: Record<string, string> = {};
  validateFieldsRecursive(fields, values, errorMap);
  const invalidFields = Object.keys(errorMap);
  return { errorMap, invalidFields };
}

/**
 * Finds accordion keys that should be open based on invalid fields
 */
export function findAccordionsWithInvalidFields(
  invalidFields: string[],
  fields: FormField[]
): string[] {
  const shouldBeOpenAccordions = new Set<string>();

  function checkFields(fieldsToCheck: FormField[]) {
    for (const field of fieldsToCheck) {
      if (field.type === FormFieldType.COLLAPSIBLE && field.collapsibleConfig) {
        // Check if any child field is invalid
        for (const childField of field.collapsibleConfig.fields) {
          if (invalidFields.includes(childField.key)) {
            shouldBeOpenAccordions.add(field.key);
            break;
          }
        }
        // Recursively check nested collapsible fields
        checkFields(field.collapsibleConfig.fields);
      }
    }
  }

  checkFields(fields);
  return Array.from(shouldBeOpenAccordions);
}

