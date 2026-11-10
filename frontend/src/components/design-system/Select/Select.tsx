import * as RadixSelect from "@radix-ui/react-select";
import { isValidElement } from "react";
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
  variant?: "outline" | "ghost";
  fullWidth?: boolean;
};

const SelectTrigger = ({
  placeholder,
  children,
  className,
  variant = "outline",
  fullWidth = false,
  ...props
}: SelectTriggerProps) => (
  <RadixSelect.Trigger
    className={[
      SelectStyles.selectTrigger,
      variant === "ghost" ? SelectStyles.selectTriggerGhost : "",
      fullWidth ? SelectStyles.selectTriggerFullWidth : "",
      className ?? "",
    ].join(" ")}
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

const SELECT_CONTENT_SIDE_OFFSET_PIXELS = 4;
const SELECT_CONTENT_COLLISION_PADDING_PIXELS = 8;

type SelectContentProps = {
  children: React.ReactNode;
};

const SelectContent = ({ children }: SelectContentProps) => (
  <RadixSelect.Portal>
    <RadixSelect.Content
      className={SelectStyles.selectContent}
      position="popper"
      side="bottom"
      align="start"
      sideOffset={SELECT_CONTENT_SIDE_OFFSET_PIXELS}
      collisionPadding={SELECT_CONTENT_COLLISION_PADDING_PIXELS}
    >
      <RadixSelect.ScrollUpButton className={SelectStyles.selectScrollButton}>
        <Icon type="chevron_up" color="grey" size="sm" />
      </RadixSelect.ScrollUpButton>
      <RadixSelect.Viewport className={SelectStyles.selectViewport}>
        {children}
      </RadixSelect.Viewport>
      <RadixSelect.ScrollDownButton className={SelectStyles.selectScrollButton}>
        <Icon type="chevron_down" color="grey" size="sm" />
      </RadixSelect.ScrollDownButton>
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
  description?: string | null;
  children: string | React.ReactNode;
};

const SelectItem = ({
  value,
  textValue,
  description,
  children,
}: SelectItemProps) => (
  <RadixSelect.Item
    value={value}
    textValue={textValue ?? (typeof children === "string" ? children : "")}
    className={SelectStyles.selectItem}
  >
    {isValidElement(children) ? (
      <RadixSelect.ItemText asChild>{children}</RadixSelect.ItemText>
    ) : (
      <RadixSelect.ItemText>{children}</RadixSelect.ItemText>
    )}
    {description && (
      <span className={SelectStyles.selectItemDescription}>{description}</span>
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
