import NavbarStyles from "src/components/Navbar/Navbar.module.css";
import { SearchBar } from "src/components/SearchBar";

export function Navbar() {
  return (
    <nav className={NavbarStyles.navbar}>
      <div>
        <SearchBar />
      </div>
    </nav>
  );
}
