import { useMemo } from "react";
import { SearchBar, Button, DropdownMenu } from "src/components/design-system";
import { TopBar } from "src/components/TopBar";
import { useHomeNavigation } from "src/components/Home/hooks";
import { createDropdownItems } from "src/components/Home/utils";
import { useTenantPermission } from "src/api/tenant/queries";
import { PERMISSIONS } from "src/constants/permissions";

export function HomeHeader() {
  const { handleCreateCanvas, handleCreateConnection } = useHomeNavigation();
  const canCreateCanvas = useTenantPermission(PERMISSIONS.CANVAS_CREATE);
  const canCreateConnection = useTenantPermission(
    PERMISSIONS.CONNECTION_CREATE,
  );

  const dropdownItems = useMemo(
    () =>
      createDropdownItems(
        handleCreateCanvas,
        handleCreateConnection,
        canCreateCanvas,
        canCreateConnection,
      ),
    [
      handleCreateCanvas,
      handleCreateConnection,
      canCreateCanvas,
      canCreateConnection,
    ],
  );

  return (
    <TopBar>
      <TopBar.Left></TopBar.Left>
      <TopBar.Center>
        <SearchBar />
      </TopBar.Center>
      <TopBar.Right>
        {dropdownItems.length > 0 && (
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
        )}
      </TopBar.Right>
    </TopBar>
  );
}
