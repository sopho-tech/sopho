import * as RadixDropdownMenu from "@radix-ui/react-dropdown-menu";
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

type DropdownMenuProps = {
  trigger: ReactNode;
  items: DropdownMenuItem[];
  align?: "start" | "center" | "end";
  side?: "top" | "right" | "bottom" | "left";
  sideOffset?: number;
  alignOffset?: number;
};

export function DropdownMenu({
  trigger,
  items,
  align = "end",
  side = "bottom",
  sideOffset = 5,
  alignOffset,
}: DropdownMenuProps) {
  return (
    <RadixDropdownMenu.Root>
      <RadixDropdownMenu.Trigger asChild>{trigger}</RadixDropdownMenu.Trigger>
      <RadixDropdownMenu.Portal>
        <RadixDropdownMenu.Content
          className={styles.content}
          align={align}
          side={side}
          sideOffset={sideOffset}
          alignOffset={alignOffset}
        >
          {items.map((item, index) => (
            <RadixDropdownMenu.Item
              key={index}
              className={styles.item}
              onClick={item.onClick}
              disabled={item.disabled}
            >
              {item.icon && (
                <Icon type={item.icon} color="default" size="sm" />
              )}
              <span className={styles.label}>{item.label}</span>
            </RadixDropdownMenu.Item>
          ))}
        </RadixDropdownMenu.Content>
      </RadixDropdownMenu.Portal>
    </RadixDropdownMenu.Root>
  );
}

