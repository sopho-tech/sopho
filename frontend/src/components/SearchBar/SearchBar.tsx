import { useEffect, useRef } from "react";
import SearchBarStyles from "src/components/SearchBar/SearchBar.module.css";
import { KEYBOARD_SHORTCUTS } from "src/constants/keyboard_shortcuts";

export function SearchBar() {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        event.key === KEYBOARD_SHORTCUTS.SEARCH.key &&
        document.activeElement?.tagName !== "INPUT"
      ) {
        event.preventDefault();
        inputRef.current?.focus();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  return (
    <div className={SearchBarStyles.searchbar}>
      <input
        ref={inputRef}
        type="text"
        placeholder="Search"
        className={SearchBarStyles.searchbarInput}
      />
    </div>
  );
}
