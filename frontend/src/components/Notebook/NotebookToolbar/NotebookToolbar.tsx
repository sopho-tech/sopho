import { useCallback, useMemo } from "react";
import { useParams } from "react-router";
import { Toolbar } from "src/components/design-system/Toolbar";
import { DropdownMenu } from "src/components/design-system/DropdownMenu";
import { Kbd } from "src/components/design-system/Kbd";
import { CellType } from "src/components/Notebook/Cell";
import {
  KEYBOARD_SHORTCUTS,
  getShortcutDisplayString,
} from "src/utils/keyboard_shortcuts";
import { useCreateCell, useReorderCell } from "src/api/cell";
import { Icon } from "src/components/design-system/Icon";
import { useStore } from "src/store";
import { Flex } from "src/components/design-system/Flex";
import styles from "src/components/Notebook/NotebookToolbar/NotebookToolbar.module.css";

export function NotebookToolbar() {
  const params = useParams();
  const canvasId = params.id || "";
  const activeNotebookId = useStore((state) => state.canvas.activeNotebookId);
  const createCellMutation = useCreateCell();
  const reorderCellMutation = useReorderCell(canvasId || undefined);

  const handleCreateNewCell = useCallback(
    (cellType: CellType) => {
      createCellMutation.mutate({
        notebook_id: activeNotebookId,
        name: null,
        content: null,
        cell_type: cellType,
      });
    },
    [activeNotebookId, createCellMutation]
  );

  const handleDeleteActiveCell = useCallback(() => {
    const activeCellId = useStore.getState().notebook.activeCellId;
    if (activeCellId) {
      useStore.getState().notebook.setCellPendingDeletion(activeCellId);
    }
  }, []);

  const handleReorderCell = useCallback(
    (movementType: "UP" | "DOWN" | "TOP" | "BOTTOM") => {
      const activeCellId = useStore.getState().notebook.activeCellId;
      if (activeCellId) {
        reorderCellMutation.mutate({ cellId: activeCellId, movementType });
      }
    },
    [reorderCellMutation]
  );

  const sxTransform = useMemo(
    () => ({ transform: "translateX(calc(-50% + 25px))" }),
    []
  );

  const handleCreateSql = useCallback(
    () => handleCreateNewCell(CellType.SQL),
    [handleCreateNewCell]
  );
  const handleCreateChart = useCallback(
    () => handleCreateNewCell(CellType.CHART),
    [handleCreateNewCell]
  );
  const handleReorderUp = useCallback(
    () => handleReorderCell("UP"),
    [handleReorderCell]
  );
  const handleReorderTop = useCallback(
    () => handleReorderCell("TOP"),
    [handleReorderCell]
  );
  const handleReorderDown = useCallback(
    () => handleReorderCell("DOWN"),
    [handleReorderCell]
  );
  const handleReorderBottom = useCallback(
    () => handleReorderCell("BOTTOM"),
    [handleReorderCell]
  );

  return (
    <Flex position="fixed" bottom={10} left="50%" sx={sxTransform} zIndex="10">
      <Toolbar className={styles.toolbar} aria-label="Notebook actions">
        <DropdownMenu>
          <DropdownMenu.Trigger>
            <Toolbar.Button className={styles.trigger} aria-label="Add cell">
              <Icon type="add" color="default" />
            </Toolbar.Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content>
            <DropdownMenu.Item onClick={handleCreateSql}>
              SQL Cell{" "}
              <Kbd>
                {getShortcutDisplayString(KEYBOARD_SHORTCUTS.ADD_SQL_CELL)}
              </Kbd>
            </DropdownMenu.Item>
            <DropdownMenu.Item onClick={handleCreateChart}>
              Chart Cell{" "}
              <Kbd>
                {getShortcutDisplayString(KEYBOARD_SHORTCUTS.ADD_CHART_CELL)}
              </Kbd>
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
            <DropdownMenu.Item onClick={handleReorderUp}>
              Move Cell Up{" "}
              <Kbd>
                {getShortcutDisplayString(KEYBOARD_SHORTCUTS.MOVE_CELL_UP)}
              </Kbd>
            </DropdownMenu.Item>
            <DropdownMenu.Item onClick={handleReorderTop}>
              Move Cell Top{" "}
              <Kbd>
                {getShortcutDisplayString(KEYBOARD_SHORTCUTS.MOVE_CELL_TOP)}
              </Kbd>
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
            <DropdownMenu.Item onClick={handleReorderDown}>
              Move Cell Down{" "}
              <Kbd>
                {getShortcutDisplayString(KEYBOARD_SHORTCUTS.MOVE_CELL_DOWN)}
              </Kbd>
            </DropdownMenu.Item>
            <DropdownMenu.Item onClick={handleReorderBottom}>
              Move Cell Bottom{" "}
              <Kbd>
                {getShortcutDisplayString(KEYBOARD_SHORTCUTS.MOVE_CELL_BOTTOM)}
              </Kbd>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu>
      </Toolbar>
    </Flex>
  );
}
