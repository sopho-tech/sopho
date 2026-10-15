import { Command } from "cmdk";
import { Icon } from "src/components/design-system/Icon";
import { IconType } from "src/components/design-system/datatypes";
import { Kbd } from "src/components/design-system/Kbd";
import {
  getShortcutDisplayString,
  KeyboardShortcut,
} from "src/utils/keyboard_shortcuts/keyboard_shortcuts";
import styles from "src/components/design-system/CommandMenu/CommandMenu.module.css";

type CommandMenuItemProps = {
  value: string;
  label: string;
  iconType?: IconType;
  shortcut?: KeyboardShortcut;
  onSelect?: (value: string) => void;
};

export const CommandMenuItem = ({
  value,
  label,
  iconType,
  shortcut,
  onSelect,
}: CommandMenuItemProps) => {
  return (
    <Command.Item value={value} onSelect={onSelect}>
      {iconType && <Icon type={iconType} color="default"></Icon>}
      {label}
      {shortcut && (
        <Kbd className={styles.shortcut}>
          {getShortcutDisplayString(shortcut)}
        </Kbd>
      )}
    </Command.Item>
  );
};
