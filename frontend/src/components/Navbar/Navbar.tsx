import "material-symbols";
import { NewAssetButton } from "src/components/NewAssetButton";
import NavbarStyles from "src/components/Navbar/Navbar.module.css";
import { SearchBar } from "src/components/SearchBar";

export function Navbar() {
  return (
    <nav className={NavbarStyles.navbar}>
      <div>
        <SearchBar />
      </div>
      <div>
        <NewAssetButton buttonText="New Asset" />
      </div>
    </nav>
  );
}
