import { FormField, FormFieldType } from "./types";
import { TextField } from "./TextField";
import { PasswordField } from "./PasswordField";
import { SelectField } from "./SelectField";
import { FormLabel } from "./FormLabel";
import { CollapsibleField } from "./CollapsibleField";
import { FieldError } from "./FieldError";
import FormStyles from "./Form.module.css";
import { AppFormReturnType } from "./hooks";

type FormFieldRendererProps = {
  field: FormField;
  form: AppFormReturnType;
  accordionState: Map<string, boolean>;
  setAccordionState: React.Dispatch<React.SetStateAction<Map<string, boolean>>>;
  fieldStyleClass?: string;
  labelStyleClass?: string;
  readonly?: boolean;
};

export function FormFieldRenderer({
  field,
  form,
  accordionState,
  setAccordionState,
  fieldStyleClass,
  labelStyleClass,
  readonly = false,
}: FormFieldRendererProps) {
  switch (field.type) {
    case FormFieldType.INPUT: {
      return (
        <form.AppField key={field.key} name={field.key}>
          {() => (
            <div className={`${FormStyles.formFieldContainer} ${fieldStyleClass || ""}`}>
              {field.showLabel !== false && (
                <FormLabel
                  infoIconToolTipMessage={field.infoIconToolTipMessage}
                  labelIconContainerStyleClass={labelStyleClass}
                >
                  {String(field.name)}
                </FormLabel>
              )}
              <TextField
                placeholder={field.placeholder}
                icon={field.icon}
                readonly={readonly}
              />
            </div>
          )}
        </form.AppField>
      );
    }
    case FormFieldType.INPUT_PASSWORD: {
      return (
        <form.AppField key={field.key} name={field.key}>
          {() => (
            <div className={`${FormStyles.formFieldContainer} ${fieldStyleClass || ""}`}>
              {field.showLabel !== false && (
                <FormLabel
                  infoIconToolTipMessage={field.infoIconToolTipMessage}
                  labelIconContainerStyleClass={labelStyleClass}
                >
                  {String(field.name)}
                </FormLabel>
              )}
              <PasswordField
                placeholder={field.placeholder}
                icon={field.icon}
                readonly={readonly}
              />
            </div>
          )}
        </form.AppField>
      );
    }
    case FormFieldType.SELECT: {
      return (
        <form.AppField key={field.key} name={field.key}>
          {(fieldState: { state: { meta: { errors: unknown[] } } }) => (
            <div className={`${FormStyles.formField} ${fieldStyleClass || ""}`}>
              {field.showLabel !== false && (
                <FormLabel
                  infoIconToolTipMessage={field.infoIconToolTipMessage}
                  labelIconContainerStyleClass={labelStyleClass}
                >
                  {String(field.name)}
                </FormLabel>
              )}
              <SelectField
                groupName={
                  typeof field.name === "string" ? field.name : "Options"
                }
                placeholderText={field.placeholder || "Select an option"}
                options={field.options || []}
                readonly={readonly}
              />
              <FieldError errors={(fieldState.state.meta.errors || []) as string[]} />
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
          labelStyleClass={labelStyleClass}
          readonly={readonly}
        />
      );
    }
    default:
      return null;
  }
}
