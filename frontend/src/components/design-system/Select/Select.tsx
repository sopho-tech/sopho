import * as RadixSelect from "@radix-ui/react-select";
import SelectStyles from "src/components/design-system/Select/Select.module.css";
import { Icon } from "src/components/design-system/Icon";

export type SelectOption = {
  label: string | React.ReactNode;
  value: string;
  textValue?: string;
};

type SelectRootProps = {
  children: React.ReactNode;
  value: string;
  onValueChange: (value: string) => void;
  disabled?: boolean;
};

const SelectRoot = ({
  children,
  value,
  onValueChange,
  disabled = false,
}: SelectRootProps) => (
  <RadixSelect.Root
    value={value || ""}
    onValueChange={onValueChange}
    disabled={disabled}
  >
    {children}
  </RadixSelect.Root>
);

SelectRoot.displayName = "Select";

type SelectTriggerProps = React.ComponentPropsWithoutRef<
  typeof RadixSelect.Trigger
> & {
  placeholder?: string;
  children?: React.ReactNode;
};

const SelectTrigger = ({
  placeholder,
  children,
  className,
  ...props
}: SelectTriggerProps) => (
  <RadixSelect.Trigger
    className={
      className
        ? `${SelectStyles.selectTrigger} ${className}`
        : SelectStyles.selectTrigger
    }
    {...props}
  >
    {children ?? (
      <>
        <RadixSelect.Value placeholder={placeholder} />
        <RadixSelect.Icon asChild>
          <div className={SelectStyles.selectIcon}>
            <Icon type="chevron_down" color="default" />
          </div>
        </RadixSelect.Icon>
      </>
    )}
  </RadixSelect.Trigger>
);

SelectTrigger.displayName = "Select.Trigger";

type SelectContentProps = {
  children: React.ReactNode;
};

const SelectContent = ({ children }: SelectContentProps) => (
  <RadixSelect.Portal>
    <RadixSelect.Content className={SelectStyles.selectContent}>
      <RadixSelect.ScrollUpButton
        className={SelectStyles.selectScrollButton}
        asChild
      >
        <Icon type="chevron_up" color="default" />
      </RadixSelect.ScrollUpButton>
      <RadixSelect.Viewport className={SelectStyles.selectViewport}>
        {children}
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
);

SelectContent.displayName = "Select.Content";

type SelectGroupProps = {
  children: React.ReactNode;
};

const SelectGroup = ({ children }: SelectGroupProps) => (
  <RadixSelect.Group>{children}</RadixSelect.Group>
);

SelectGroup.displayName = "Select.Group";

type SelectLabelProps = {
  children: React.ReactNode;
};

const SelectLabel = ({ children }: SelectLabelProps) => (
  <RadixSelect.Label className={SelectStyles.selectLabel}>
    {children}
  </RadixSelect.Label>
);

SelectLabel.displayName = "Select.Label";

type SelectItemProps = {
  value: string;
  textValue?: string;
  children: string | React.ReactNode;
};

const SelectItem = ({ value, textValue, children }: SelectItemProps) => (
  <RadixSelect.Item
    value={value}
    textValue={textValue ?? (typeof children === "string" ? children : "")}
    className={SelectStyles.selectItem}
  >
    {typeof children === "string" ? (
      <RadixSelect.ItemText>{children}</RadixSelect.ItemText>
    ) : (
      <RadixSelect.ItemText asChild>{children}</RadixSelect.ItemText>
    )}
    <RadixSelect.ItemIndicator className={SelectStyles.selectItemIndicator}>
      <Icon size="sm" type="check" color="default" />
    </RadixSelect.ItemIndicator>
  </RadixSelect.Item>
);

SelectItem.displayName = "Select.Item";

export const Select = Object.assign(SelectRoot, {
  Trigger: SelectTrigger,
  Content: SelectContent,
  Group: SelectGroup,
  Label: SelectLabel,
  Item: SelectItem,
});
