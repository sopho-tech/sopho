import * as Select from "@radix-ui/react-select";
import * as Toolbar from "@radix-ui/react-toolbar";
import SophoSelectStyles from "src/components/SophoSelect/SophoSelect.module.css";
import { Icon } from "src/components/design-system/Icon";

type SophoSelectProps = {
  groupName: string;
  initialValue:
    | {
        label: string;
        value: string;
      }
    | undefined;
  onValueChange: (value: string) => void;
  options:
    | {
        label: string;
        value: string;
      }[]
    | undefined;
};

export function SophoSelect({
  groupName,
  initialValue,
  onValueChange,
  options,
}: SophoSelectProps) {
  const selectItems = options?.map((option) => (
    <Select.Item
      key={option.value}
      value={option.value}
      textValue={option.label}
      className={SophoSelectStyles.selectItem}
    >
      <Select.ItemText>{option.label}</Select.ItemText>
      <Select.ItemIndicator className={SophoSelectStyles.selectItemIndicator}>
        <Icon type="check" color="default" />
      </Select.ItemIndicator>
    </Select.Item>
  ));

  const selectBody = (
    <>
      <Toolbar.Button asChild className={SophoSelectStyles.toolbarButton}>
        <Select.Trigger className={SophoSelectStyles.selectTrigger}>
          <Select.Value
            className={SophoSelectStyles.selectValue}
            placeholder="Select a connection"
          >
            {initialValue?.label}
          </Select.Value>
          <Select.Icon className={SophoSelectStyles.selectIcon} asChild>
            <div>
              <Icon type="chevron_down" color="default" />
            </div>
          </Select.Icon>
        </Select.Trigger>
      </Toolbar.Button>
      <Select.Portal>
        <Select.Content className={SophoSelectStyles.selectContent}>
          <Select.ScrollUpButton
            className={SophoSelectStyles.selectScrollButton}
            asChild
          >
            <Icon type="chevron_up" color="default" />
          </Select.ScrollUpButton>
          <Select.Viewport className={SophoSelectStyles.selectViewport}>
            <Select.Group>
              <Select.Label className={SophoSelectStyles.selectLabel}>
                {groupName}
              </Select.Label>
              {selectItems}
            </Select.Group>
          </Select.Viewport>
          <Select.ScrollDownButton
            className={SophoSelectStyles.selectScrollButton}
            asChild
          >
            <Icon type="chevron_down" color="default" />
          </Select.ScrollDownButton>
          <Select.Arrow />
        </Select.Content>
      </Select.Portal>
    </>
  );

  return (
    <Select.Root
      value={initialValue?.value || ""}
      onValueChange={onValueChange}
    >
      {selectBody}
    </Select.Root>
  );
}
