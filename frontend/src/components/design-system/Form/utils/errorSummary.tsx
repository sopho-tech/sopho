import {
  FormField,
  FormFieldType,
} from "src/components/design-system/Form/types";

export function getErrorSummary(
  formFieldMeta: any,
  fields: FormField[]
): string | undefined {
  let invalidCount = 0;

  const checkField = (field: FormField) => {
    const fieldMeta = formFieldMeta[field.key];
    if (fieldMeta && fieldMeta.isValid === false) {
      invalidCount++;
    }

    if (field.type === FormFieldType.COLLAPSIBLE && field.collapsibleConfig) {
      field.collapsibleConfig.fields.forEach(checkField);
    }
  };

  fields.forEach(checkField);

  if (invalidCount > 0) {
    return `${invalidCount} invalid field${invalidCount === 1 ? "" : "s"} need${invalidCount === 1 ? "s" : ""} to be fixed below`;
  }

  return undefined;
}
