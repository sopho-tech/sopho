import { FormField } from "./types";
import type { AccordionItemConfig } from "src/components/design-system/Accordion/Accordion";
import { Accordion } from "src/components/design-system/Accordion";
import { FormFieldRenderer } from "./FormFieldRenderer";

type CollapsibleFieldProps = {
  field: FormField;
  form: any;
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
  if (!field.collapsibleConfig) {
    return null;
  }

  const items: AccordionItemConfig[] = [
    {
      value: field.key,
      trigger: field.name,
      content: field.collapsibleConfig.fields.map((childField) => (
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
      )),
    },
  ];

  const accordionValues = Array.from(accordionState.entries())
    .filter(([_, isOpen]) => isOpen)
    .map(([key]) => key);

  return (
    <Accordion
      items={items}
      value={accordionValues}
      onValueChange={(newValues) => {
        const newState = new Map(accordionState);
        newState.set(field.key, newValues.includes(field.key));
        setAccordionState(newState);
      }}
    />
  );
}
