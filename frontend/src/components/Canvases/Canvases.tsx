import { CanvasCreateDialog } from "src/components/Canvases/CanvasCreateDialog";
import { Flex, Heading, SearchBar, Button } from "src/components/design-system";
import { useCanvasActions } from "src/components/Canvases/hooks.tsx";
import { CanvasesTable } from "src/components/Canvases/CanvasesTable";
import { TopBar } from "src/components/TopBar";

export function Canvases() {
  const { handleOpenCreateDialog } = useCanvasActions();

  return (
    <Flex direction="column" flex="grow" gap="lg" overflow="scrollY">
      <TopBar>
        <TopBar.Left></TopBar.Left>
        <TopBar.Center>
          <SearchBar />
        </TopBar.Center>
        <TopBar.Right>
          <Button
            leadingIconName="add"
            shape="rectangle"
            label="New"
            onClick={handleOpenCreateDialog}
            backgroundColor="accent"
            size="md"
          />
        </TopBar.Right>
      </TopBar>
      <Flex paddingX="2xl" direction="column" gap="lg">
        <Heading accessbilityLevel={1}>Canvases</Heading>
        <CanvasesTable />
      </Flex>
      <CanvasCreateDialog />
    </Flex>
  );
}
