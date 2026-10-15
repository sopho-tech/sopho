import {
  FormField,
  FormFieldType,
} from "src/components/design-system/Form/types";
import { TextField } from "src/components/design-system/Form/TextField";
import { PasswordField } from "src/components/design-system/Form/PasswordField";
import { SelectField } from "src/components/design-system/Form/SelectField";
import { FormLabel } from "src/components/design-system/Form/FormLabel";
import { CollapsibleField } from "src/components/design-system/Form/CollapsibleField";
import { FieldError } from "src/components/design-system/Form/FieldError";
import FormStyles from "src/components/design-system/Form/Form.module.css";
import { AppFormReturnType } from "src/components/design-system/Form/hooks";
import { FormFieldIdContext } from "src/components/design-system/Form/form-context";
import { useId } from "react";

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
  const formFieldId = useId();
  switch (field.type) {
    case FormFieldType.INPUT: {
      return (
        <FormFieldIdContext value={formFieldId}>
          <form.AppField key={field.key} name={field.key}>
            {() => (
              <div
                className={`${FormStyles.formFieldContainer} ${fieldStyleClass || ""}`}
              >
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
        </FormFieldIdContext>
      );
    }
    case FormFieldType.INPUT_PASSWORD: {
      return (
        <FormFieldIdContext value={formFieldId}>
          <form.AppField key={field.key} name={field.key}>
            {() => (
              <div
                className={`${FormStyles.formFieldContainer} ${fieldStyleClass || ""}`}
              >
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
        </FormFieldIdContext>
      );
    }
    case FormFieldType.SELECT: {
      return (
        <FormFieldIdContext value={formFieldId}>
          <form.AppField key={field.key} name={field.key}>
            {(fieldState: { state: { meta: { errors: unknown[] } } }) => (
              <div
                className={`${FormStyles.formField} ${fieldStyleClass || ""}`}
              >
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
                <FieldError
                  errors={(fieldState.state.meta.errors || []) as string[]}
                />
              </div>
            )}
          </form.AppField>
        </FormFieldIdContext>
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
