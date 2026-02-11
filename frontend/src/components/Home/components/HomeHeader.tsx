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
        <DropdownMenu>
          <DropdownMenu.Trigger>
            <Button
              label="New"
              leadingIconName="add"
              backgroundColor="accent"
              shape="rectangle"
              size="md"
            />
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end" side="bottom">
            {dropdownItems.map((item, index) => (
              <DropdownMenu.Item
                key={index}
                icon={item.icon}
                onClick={item.onClick}
                disabled={item.disabled}
              >
                {item.label}
              </DropdownMenu.Item>
            ))}
          </DropdownMenu.Content>
        </DropdownMenu>
      </TopBar.Right>
    </TopBar>
  );
}
