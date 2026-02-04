import { useMemo } from "react";
import { SearchBar, Button, DropdownMenu } from "src/components/design-system";
import { TopBar } from "src/components/TopBar";
import { useHomeNavigation } from "../hooks";
import { createDropdownItems } from "../utils";

export function HomeHeader() {
  const { handleCreateCanvas, handleCreateConnection } = useHomeNavigation();

  const dropdownItems = useMemo(
    () => createDropdownItems(handleCreateCanvas, handleCreateConnection),
    [handleCreateCanvas, handleCreateConnection]
  );

  return (
    <TopBar>
      <TopBar.Left></TopBar.Left>
      <TopBar.Center>
        <SearchBar />
      </TopBar.Center>
      <TopBar.Right>
        <DropdownMenu
          trigger={
            <Button
              label="New"
              leadingIconName="add"
              backgroundColor="accent"
              shape="rectangle"
              size="md"
            />
          }
          items={dropdownItems}
          align="end"
          side="bottom"
        />
      </TopBar.Right>
    </TopBar>
  );
}
