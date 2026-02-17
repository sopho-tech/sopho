import { useCallback } from "react";
import { FormField } from "./types";
import type { AppFormReturnType } from "./hooks";
import { Accordion } from "src/components/design-system/Accordion";
import { FormFieldRenderer } from "./FormFieldRenderer";

type CollapsibleFieldProps = {
  field: FormField;
  form: AppFormReturnType;
  accordionState: Map<string, boolean>;
  setAccordionState: React.Dispatch<React.SetStateAction<Map<string, boolean>>>;
  fieldStyleClass?: string;
  labelStyleClass?: string;
  readonly?: boolean;
};

export function CollapsibleField({
  field,
  form,
  accordionState,
  setAccordionState,
  fieldStyleClass,
  labelStyleClass,
  readonly = false,
}: CollapsibleFieldProps) {
  const handleValueChange = useCallback(
    (newValues: string[]) => {
      const newState = new Map(accordionState);
      newState.set(field.key, newValues.includes(field.key));
      setAccordionState(newState);
    },
    [accordionState, field.key, setAccordionState]
  );

  if (!field.collapsibleConfig) {
    return null;
  }

  const accordionValues = Array.from(accordionState.entries())
    .filter(([, isOpen]) => isOpen)
    .map(([key]) => key);

  return (
    <Accordion
      value={accordionValues}
      onValueChange={handleValueChange}
    >
      <Accordion.Item value={field.key}>
        <Accordion.Trigger>{field.name}</Accordion.Trigger>
        <Accordion.Content>
          {field.collapsibleConfig.fields.map((childField) => (
            <FormFieldRenderer
              key={childField.key}
              field={childField}
              form={form}
              accordionState={accordionState}
              setAccordionState={setAccordionState}
              fieldStyleClass={fieldStyleClass}
              labelStyleClass={labelStyleClass}
              readonly={readonly}
            />
          ))}
        </Accordion.Content>
      </Accordion.Item>
    </Accordion>
  );
}
