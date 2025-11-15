import { NotebookCreateDialog } from "src/components/Notebooks/NotebookCreateDialog";
import { Flex } from "src/components/design-system/Flex/Flex";
import { Heading } from "src/components/design-system";
import { useNotebookActions } from "src/components/Notebooks/hooks.tsx";
import { NotebooksTable } from "src/components/Notebooks/NotebooksTable";
import { Button } from "../design-system/Button/Button";

export function Notebooks() {
  const {
    handleViewNotebook,
    handleEditNotebook,
    handleDeleteNotebook,
    handleOpenCreateDialog,
  } = useNotebookActions();

  return (
    <Flex direction="column" flex="grow" paddingX="lg" paddingY="lg">
      <Flex
        direction="row"
        justifyContent="space-between"
        marginBottom="lg"
        alignItems="center"
      >
        <Heading accessbilityLevel={1}>Notebooks</Heading>
        <Button
          leadingIconName="add"
          shape="rectangle"
          label="New Notebook"
          onClick={handleOpenCreateDialog}
          backgroundColor="accent"
          size="md"
        />
        <NotebookCreateDialog />
      </Flex>
      <NotebooksTable
        onViewClick={handleViewNotebook}
        onEditClick={handleEditNotebook}
        onDeleteClick={handleDeleteNotebook}
      />
    </Flex>
  );
}
