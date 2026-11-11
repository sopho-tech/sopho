import { CanvasCreateDialog } from "src/components/Canvases/CanvasCreateDialog";
import { Flex, SearchBar, Button } from "src/components/design-system";
import { useCanvasActions } from "src/components/Canvases/hooks.tsx";
import { CanvasesTable } from "src/components/Canvases/CanvasesTable";
import { TopBar } from "src/components/TopBar";
import { useTenantPermission } from "src/api/tenant/queries";
import { PERMISSIONS } from "src/constants/permissions";

export function Canvases() {
  const { handleOpenCreateDialog } = useCanvasActions();
  const canCreateCanvas = useTenantPermission(PERMISSIONS.CANVAS_CREATE);

  return (
    <Flex direction="column" flex="grow" gap="lg" overflow="scrollY">
      <TopBar>
        <TopBar.Left></TopBar.Left>
        <TopBar.Center>
          <SearchBar />
        </TopBar.Center>
        <TopBar.Right>
          {canCreateCanvas && (
            <Button
              leadingIconName="add"
              shape="rectangle"
              label="New"
              onClick={handleOpenCreateDialog}
              backgroundColor="accent"
              size="md"
            />
          )}
        </TopBar.Right>
      </TopBar>
      <Flex
        paddingX="2xl"
        direction="column"
        gap="lg"
        flex="grow"
        overflow="scrollY"
      >
        <CanvasesTable />
      </Flex>
      <CanvasCreateDialog />
    </Flex>
  );
}
