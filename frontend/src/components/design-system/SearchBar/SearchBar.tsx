import { useState } from "react";
import { Input } from "src/components/design-system/Input/Input";
import { Kbd } from "src/components/design-system/Kbd";
import {
  KEYBOARD_SHORTCUTS,
  getShortcutDisplayString,
} from "src/utils/keyboard_shortcuts";
import { useStore } from "src/store";
import styles from "./SearchBar.module.css";

export function SearchBar() {
  const [value, setValue] = useState("");
  const setIsCommandMenuOpen = useStore((state) => state.commandMenu.setIsOpen);

  const openCommandMenu = () => {
    setIsCommandMenuOpen(true);
  };

  return (
    <Input
      value={value}
      onChange={(e) => setValue(e.target.value)}
      onClick={openCommandMenu}
      onFocus={openCommandMenu}
      type="text"
      placeholder="Search"
      leadingIcon="search"
      laggingElement={<Kbd>{getShortcutDisplayString(KEYBOARD_SHORTCUTS.OPEN_COMMAND_MENU)}</Kbd>}
      className={styles.container}
    />
  );
}
