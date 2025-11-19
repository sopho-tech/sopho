import { FormField, FormFieldType } from "./types";
import { TextField } from "./TextField";
import { SelectField } from "./SelectField";
import { CollapsibleField } from "./CollapsibleField";
import { FieldError } from "./FieldError";
import FormStyles from "./Form.module.css";

type FormFieldRendererProps = {
  field: FormField;
  form: any;
  accordionState: Map<string, boolean>;
  setAccordionState: React.Dispatch<React.SetStateAction<Map<string, boolean>>>;
  fieldStyleClass?: string;
};

export function FormFieldRenderer({
  field,
  form,
  accordionState,
  setAccordionState,
  fieldStyleClass,
}: FormFieldRendererProps) {
  switch (field.type) {
    case FormFieldType.INPUT: {
      return (
        <form.AppField key={field.key} name={field.key}>
          {(fieldState: any) => (
            <div className={`${FormStyles.formField} ${fieldStyleClass || ""}`}>
              <TextField label={String(field.name)} />
              <FieldError errors={fieldState.state.meta.errors} />
            </div>
          )}
        </form.AppField>
      );
    }
    case FormFieldType.SELECT: {
      return (
        <form.AppField key={field.key} name={field.key}>
          {(fieldState: any) => (
            <div className={`${FormStyles.formField} ${fieldStyleClass || ""}`}>
              <SelectField
                label={String(field.name)}
                groupName={
                  typeof field.name === "string" ? field.name : "Options"
                }
                placeholderText={field.placeholder || "Select an option"}
                options={field.options || []}
              />
              <FieldError errors={fieldState.state.meta.errors} />
            </div>
          )}
        </form.AppField>
      );
    }
    case FormFieldType.COLLAPSIBLE: {
      return (
        <CollapsibleField
          key={field.key}
          field={field}
          form={form}
          accordionState={accordionState}
          setAccordionState={setAccordionState}
          fieldStyleClass={fieldStyleClass}
        />
      );
    }
    default:
      return null;
  }
}

