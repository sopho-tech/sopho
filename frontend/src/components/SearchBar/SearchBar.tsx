import { useRef } from "react";
import SearchBarStyles from "src/components/SearchBar/SearchBar.module.css";

export function SearchBar() {
  const inputRef = useRef<HTMLInputElement>(null);
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
