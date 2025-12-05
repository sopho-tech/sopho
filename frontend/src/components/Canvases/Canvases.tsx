import { CanvasCreateDialog } from "src/components/Canvases/CanvasCreateDialog";
import { Flex } from "src/components/design-system/Flex/Flex";
import { Heading } from "src/components/design-system";
import { useCanvasActions } from "src/components/Canvases/hooks.tsx";
import { CanvasesTable } from "src/components/Canvases/CanvasesTable";
import { Button } from "src/components/design-system";

export function Canvases() {
  const {
    handleViewCanvas,
    handleEditCanvas,
    handleDeleteCanvas,
    handleOpenCreateDialog,
  } = useCanvasActions();

  return (
    <Flex direction="column" flex="grow" paddingX="lg" paddingY="lg">
      <Flex
        direction="row"
        justifyContent="space-between"
        marginBottom="lg"
        alignItems="center"
      >
        <Heading accessbilityLevel={1}>Canvases</Heading>
        <Button
          leadingIconName="add"
          shape="rectangle"
          label="New Canvas"
          onClick={handleOpenCreateDialog}
          backgroundColor="accent"
          size="md"
        />
        <CanvasCreateDialog />
      </Flex>
      <CanvasesTable
        onViewClick={handleViewCanvas}
        onEditClick={handleEditCanvas}
        onDeleteClick={handleDeleteCanvas}
      />
    </Flex>
  );
}
