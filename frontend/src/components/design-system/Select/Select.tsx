import * as RadixSelect from "@radix-ui/react-select";
import SelectStyles from "src/components/design-system/Select/Select.module.css";
import { Icon } from "src/components/design-system/Icon";

export type SelectProps = {
  placeholderText: string;
  groupName: string;
  options: {
    label: string;
    value: string;
  }[];
  value: string;
  onValueChange: (value: string) => void;
};

export function Select({
  placeholderText,
  groupName,
  options,
  value,
  onValueChange,
}: SelectProps) {
  const selectedOption = options.find((option) => option.value === value);
  const selectItems = options?.map((option) => (
    <RadixSelect.Item
      key={option.value}
      value={option.value}
      textValue={option.label}
      className={SelectStyles.selectItem}
    >
      <RadixSelect.ItemText>{option.label}</RadixSelect.ItemText>
      <RadixSelect.ItemIndicator className={SelectStyles.selectItemIndicator}>
        <Icon type="check" color="default" />
      </RadixSelect.ItemIndicator>
    </RadixSelect.Item>
  ));
  return (
    <RadixSelect.Root value={value || ""} onValueChange={onValueChange}>
      <RadixSelect.Trigger className={SelectStyles.selectTrigger}>
        <RadixSelect.Value placeholder={placeholderText}>
          {selectedOption?.label}
        </RadixSelect.Value>
        <RadixSelect.Icon asChild>
          <div className={SelectStyles.selectIcon}>
            <Icon type="chevron_down" color="default" />
          </div>
        </RadixSelect.Icon>
      </RadixSelect.Trigger>
      <RadixSelect.Portal>
        <RadixSelect.Content className={SelectStyles.selectContent}>
          <RadixSelect.ScrollUpButton
            className={SelectStyles.selectScrollButton}
            asChild
          >
            <Icon type="chevron_up" color="default" />
          </RadixSelect.ScrollUpButton>
          <RadixSelect.Viewport className={SelectStyles.selectViewport}>
            <RadixSelect.Group>
              <RadixSelect.Label className={SelectStyles.selectLabel}>
                {groupName}
              </RadixSelect.Label>
              {selectItems}
            </RadixSelect.Group>
          </RadixSelect.Viewport>
          <RadixSelect.ScrollDownButton
            className={SelectStyles.selectScrollButton}
            asChild
          >
            <Icon type="chevron_down" color="default" />
          </RadixSelect.ScrollDownButton>
          <RadixSelect.Arrow />
        </RadixSelect.Content>
      </RadixSelect.Portal>
    </RadixSelect.Root>
  );
}
