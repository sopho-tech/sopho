import { useParams } from "react-router";
import { Toolbar } from "src/components/design-system/Toolbar";
import { DropdownMenu } from "src/components/design-system/DropdownMenu";
import { Kbd } from "src/components/design-system/Kbd";
import { CellType } from "src/components/Notebook/Cell";
import { useCreateCell, useDeleteCell, useReorderCell } from "src/api/cell";
import { Icon } from "src/components/design-system/Icon";
import { useStore } from "src/store";
import { Flex } from "src/components/design-system/Flex";
import styles from "src/components/Notebook/NotebookToolbar/NotebookToolbar.module.css";

export function NotebookToolbar() {
  const params = useParams();
  const canvasId = params.id || "";
  const activeNotebookId = useStore((state) => state.canvas.activeNotebookId);
  const createCellMutation = useCreateCell();
  const deleteCellMutation = useDeleteCell(
    activeNotebookId ?? undefined,
    canvasId || undefined
  );
  const reorderCellMutation = useReorderCell(canvasId || undefined);

  const handleCreateNewCell = (cellType: CellType) => {
    createCellMutation.mutate({
      notebook_id: activeNotebookId,
      name: null,
      content: null,
      cell_type: cellType,
    });
  };

  const handleDeleteActiveCell = () => {
    const activeCellId = useStore.getState().notebook.activeCellId;
    if (activeCellId) {
      deleteCellMutation.mutate(activeCellId, {
        onSuccess: () => {
          useStore.getState().notebook.setActiveCellId("");
        },
      });
    }
  };

  const handleReorderCell = (
    movementType: "UP" | "DOWN" | "TOP" | "BOTTOM"
  ) => {
    const activeCellId = useStore.getState().notebook.activeCellId;
    if (activeCellId) {
      reorderCellMutation.mutate({ cellId: activeCellId, movementType });
    }
  };

  return (
    <Flex
      position="fixed"
      bottom={10}
      right="50%"
      sx={{ transform: "translateX(50%)" }}
      zIndex="10"
    >
      <Toolbar className={styles.toolbar} aria-label="Notebook actions">
        <DropdownMenu>
          <DropdownMenu.Trigger>
            <Toolbar.Button className={styles.trigger} aria-label="Add cell">
              <Icon type="add" color="default" />
            </Toolbar.Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.Item
              onClick={() => handleCreateNewCell(CellType.MARKDOWN)}
            >
              Markdown Cell <Kbd>⌘ M</Kbd>
            </DropdownMenu.Item>
            <DropdownMenu.Item
              onClick={() => handleCreateNewCell(CellType.SQL)}
            >
              SQL Cell <Kbd>⌘ Q</Kbd>
            </DropdownMenu.Item>
            <DropdownMenu.Item
              onClick={() => handleCreateNewCell(CellType.CHART)}
            >
              Chart Cell <Kbd>⌘ C</Kbd>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu>
        <Toolbar.Button
          className={styles.trigger}
          aria-label="Delete cell"
          onClick={handleDeleteActiveCell}
        >
          <Icon type="remove" color="default" />
        </Toolbar.Button>
        <DropdownMenu>
          <DropdownMenu.Trigger>
            <Toolbar.Button
              className={styles.trigger}
              aria-label="Move cell up"
            >
              <Icon type="arrow_up" color="default" />
            </Toolbar.Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.Item onClick={() => handleReorderCell("UP")}>
              Move Cell Up <Kbd>⌘ M</Kbd>
            </DropdownMenu.Item>
            <DropdownMenu.Item onClick={() => handleReorderCell("TOP")}>
              Move Cell Top <Kbd>⇧ ⌘ M</Kbd>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu>
        <DropdownMenu>
          <DropdownMenu.Trigger>
            <Toolbar.Button
              className={styles.trigger}
              aria-label="Move cell down"
            >
              <Icon type="arrow_down" color="default" />
            </Toolbar.Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.Item onClick={() => handleReorderCell("DOWN")}>
              Move Cell Down <Kbd>⌘ T</Kbd>
            </DropdownMenu.Item>
            <DropdownMenu.Item onClick={() => handleReorderCell("BOTTOM")}>
              Move Cell Bottom <Kbd>⇧ ⌘ T</Kbd>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu>
      </Toolbar>
    </Flex>
  );
}
