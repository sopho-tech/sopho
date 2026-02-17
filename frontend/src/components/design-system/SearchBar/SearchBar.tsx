import { Fragment, useState } from "react";
import { Input } from "src/components/design-system/Input/Input";
import { Kbd } from "src/components/design-system/Kbd";
import { CommandMenu } from "src/components/design-system/CommandMenu/CommandMenu";
import {
  KEYBOARD_SHORTCUTS,
  getShortcutDisplayString,
} from "src/utils/keyboard_shortcuts";
import styles from "./SearchBar.module.css";

export function SearchBar() {
  const [value, setValue] = useState("");
  const [isCommandMenuOpen, setIsCommandMenuOpen] = useState(false);

  const handleInputClick = () => {
    setIsCommandMenuOpen(true);
  };

  const handleInputFocus = () => {
    setIsCommandMenuOpen(true);
  };

  return (
    <Fragment>
      <Input
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onClick={handleInputClick}
        onFocus={handleInputFocus}
        type="text"
        placeholder="Search"
        leadingIcon="search"
        laggingElement={<Kbd>{getShortcutDisplayString(KEYBOARD_SHORTCUTS.OPEN_COMMAND_MENU)}</Kbd>}
        className={styles.container}
      />
      <CommandMenu
        open={isCommandMenuOpen}
        onOpenChange={setIsCommandMenuOpen}
      />
    </Fragment>
  );
}
