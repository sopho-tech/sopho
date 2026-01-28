import * as RadixSelect from "@radix-ui/react-select";
import SelectStyles from "src/components/design-system/Select/Select.module.css";
import { Icon } from "src/components/design-system/Icon";

export type SelectOption = {
  label: string | React.ReactNode;
  value: string;
  textValue?: string;
};

export type SelectProps = {
  placeholderText: string;
  groupName: string;
  options: SelectOption[];
  value: string;
  onValueChange: (value: string) => void;
  disabled?: boolean;
};

export function Select({
  placeholderText,
  groupName,
  options,
  value,
  onValueChange,
  disabled = false,
}: SelectProps) {
  const selectedOption = options.find((option) => option.value === value);

  const getTextValue = (option: SelectOption): string => {
    if (option.textValue) {
      return option.textValue;
    }
    if (typeof option.label === "string") {
      return option.label;
    }
    return "";
  };

  const selectItems = options?.map((option) => (
    <RadixSelect.Item
      key={option.value}
      value={option.value}
      textValue={getTextValue(option)}
      className={SelectStyles.selectItem}
    >
      {typeof option.label === "string" ? (
        <RadixSelect.ItemText>{option.label}</RadixSelect.ItemText>
      ) : (
        <RadixSelect.ItemText asChild>{option.label}</RadixSelect.ItemText>
      )}
      <RadixSelect.ItemIndicator className={SelectStyles.selectItemIndicator}>
        <Icon size="sm" type="check" color="default" />
      </RadixSelect.ItemIndicator>
    </RadixSelect.Item>
  ));
  return (
    <RadixSelect.Root value={value || ""} onValueChange={onValueChange} disabled={disabled}>
      <RadixSelect.Trigger className={SelectStyles.selectTrigger} disabled={disabled}>
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
