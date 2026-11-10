import * as RadixDropdownMenu from "@radix-ui/react-dropdown-menu";
import classNames from "classnames";
import { ReactNode } from "react";
import { IconType } from "src/components/design-system/datatypes";
import { Icon } from "src/components/design-system/Icon";
import styles from "src/components/design-system/DropdownMenu/DropdownMenu.module.css";

export type DropdownMenuItem = {
  icon?: IconType;
  label: string;
  onClick: () => void;
  disabled?: boolean;
};

export type DropdownMenuRootProps = {
  children?: ReactNode;
  dir?: "ltr" | "rtl";
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  modal?: boolean;
};

export function DropdownMenu({ children, ...props }: DropdownMenuRootProps) {
  return <RadixDropdownMenu.Root {...props}>{children}</RadixDropdownMenu.Root>;
}

type DropdownMenuTriggerProps = {
  children: ReactNode;
  asChild?: boolean;
};

function DropdownMenuTrigger({
  children,
  asChild = true,
}: DropdownMenuTriggerProps) {
  return (
    <RadixDropdownMenu.Trigger asChild={asChild}>
      {children}
    </RadixDropdownMenu.Trigger>
  );
}

type DropdownMenuContentProps = {
  children: ReactNode;
  align?: "start" | "center" | "end";
  side?: "top" | "right" | "bottom" | "left";
  sideOffset?: number;
  alignOffset?: number;
  preventAutoFocus?: boolean;
  minWidthOfTrigger?: boolean;
};

function DropdownMenuContent({
  children,
  align = "end",
  side = "bottom",
  sideOffset = 5,
  alignOffset,
  preventAutoFocus = false,
  minWidthOfTrigger = false,
}: DropdownMenuContentProps) {
  const autoFocusProps = preventAutoFocus
    ? {
        onOpenAutoFocus: (event: Event) => event.preventDefault(),
        onCloseAutoFocus: (event: Event) => event.preventDefault(),
      }
    : {};
  return (
    <RadixDropdownMenu.Portal>
      <RadixDropdownMenu.Content
        className={classNames(styles.content, {
          [styles.minWidthOfTrigger]: minWidthOfTrigger,
        })}
        align={align}
        side={side}
        sideOffset={sideOffset}
        alignOffset={alignOffset}
        onClick={(event) => event.stopPropagation()}
        {...autoFocusProps}
      >
        {children}
      </RadixDropdownMenu.Content>
    </RadixDropdownMenu.Portal>
  );
}

type DropdownMenuItemProps = {
  icon?: IconType;
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
};

function DropdownMenuItemComponent({
  icon,
  children,
  onClick,
  disabled,
}: DropdownMenuItemProps) {
  return (
    <RadixDropdownMenu.Item
      className={styles.item}
      onClick={onClick}
      disabled={disabled}
    >
      {icon && <Icon type={icon} color="default" size="sm" />}
      <span className={styles.label}>{children}</span>
    </RadixDropdownMenu.Item>
  );
}

type DropdownMenuItemWithDescriptionProps = {
  label: string;
  description: string;
  onClick: () => void;
  highlighted?: boolean;
  disabled?: boolean;
};

function DropdownMenuItemWithDescriptionComponent({
  label,
  description,
  onClick,
  highlighted,
  disabled,
}: DropdownMenuItemWithDescriptionProps) {
  return (
    <RadixDropdownMenu.Item
      className={classNames(styles.itemWithDescription, {
        [styles.highlighted]: highlighted,
      })}
      onClick={onClick}
      disabled={disabled}
    >
      <span className={styles.itemLabel}>{label}</span>
      <span className={styles.itemDescription}>{description}</span>
    </RadixDropdownMenu.Item>
  );
}

type DropdownMenuRadioGroupProps = {
  value: string;
  onValueChange: (value: string) => void;
  children: ReactNode;
};

function DropdownMenuRadioGroup({
  value,
  onValueChange,
  children,
}: DropdownMenuRadioGroupProps) {
  return (
    <RadixDropdownMenu.RadioGroup value={value} onValueChange={onValueChange}>
      {children}
    </RadixDropdownMenu.RadioGroup>
  );
}

type DropdownMenuRadioItemProps = {
  value: string;
  children: ReactNode;
};

function DropdownMenuRadioItem({
  value,
  children,
}: DropdownMenuRadioItemProps) {
  return (
    <RadixDropdownMenu.RadioItem className={styles.item} value={value}>
      <span className={styles.label}>{children}</span>
      <RadixDropdownMenu.ItemIndicator className={styles.indicator}>
        <Icon type="check" color="accent" size="sm" />
      </RadixDropdownMenu.ItemIndicator>
    </RadixDropdownMenu.RadioItem>
  );
}

function DropdownMenuSeparator() {
  return <RadixDropdownMenu.Separator className={styles.separator} />;
}

DropdownMenu.Trigger = DropdownMenuTrigger;
DropdownMenu.Content = DropdownMenuContent;
DropdownMenu.Item = DropdownMenuItemComponent;
DropdownMenu.ItemWithDescription = DropdownMenuItemWithDescriptionComponent;
DropdownMenu.RadioGroup = DropdownMenuRadioGroup;
DropdownMenu.RadioItem = DropdownMenuRadioItem;
DropdownMenu.Separator = DropdownMenuSeparator;
