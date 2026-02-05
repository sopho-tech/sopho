import { FormField, FormFieldType } from "./types";
import { TextField } from "./TextField";
import { PasswordField } from "./PasswordField";
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
            <div className={`${FormStyles.formFieldContainer}`}>
              <TextField
                label={String(field.name)}
                placeholder={field.placeholder}
                showLabel={field.showLabel}
                icon={field.icon}
                readonly={readonly}
                containerStyleClass={fieldStyleClass}
                labelStyleClass={labelStyleClass}
              />
            </div>
          )}
        </form.AppField>
      );
    }
    case FormFieldType.INPUT_PASSWORD: {
      return (
        <form.AppField key={field.key} name={field.key}>
          {(fieldState: any) => (
            <div className={`${FormStyles.formField} ${fieldStyleClass || ""}`}>
              <PasswordField
                label={String(field.name)}
                placeholder={field.placeholder}
                showLabel={field.showLabel}
                icon={field.icon}
                readonly={readonly}
                containerStyleClass={fieldStyleClass}
                labelStyleClass={labelStyleClass}
              />
              {/* <FieldError errors={fieldState.state.meta.errors} /> */}
            </div>
          )}
        </form.AppField>
      );
    }
    case FormFieldType.SELECT: {
      return (
        <form.AppField key={field.key} name={field.key} className>
          {(fieldState: any) => (
            <div className={`${FormStyles.formField} ${fieldStyleClass || ""}`}>
              <SelectField
                label={String(field.name)}
                groupName={
                  typeof field.name === "string" ? field.name : "Options"
                }
                placeholderText={field.placeholder || "Select an option"}
                options={field.options || []}
                infoIconToolTipMessage={field.infoIconToolTipMessage}
                showLabel={field.showLabel}
                readonly={readonly}
                labelStyleClass={labelStyleClass}
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
          labelStyleClass={labelStyleClass}
          readonly={readonly}
        />
      );
    }
    default:
      return null;
  }
}
