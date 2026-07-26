import { Command } from "cmdk";
import { Icon } from "../Icon";
import { IconType } from "../datatypes";
import { Kbd } from "../Kbd";
import {
  getShortcutDisplayString,
  KeyboardShortcut,
} from "src/utils/keyboard_shortcuts/keyboard_shortcuts";
import styles from "./CommandMenu.module.css";

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
