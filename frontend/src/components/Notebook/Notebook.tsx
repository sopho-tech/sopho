import { Cell } from "src/components/Notebook/Cell";
import { ChartCell } from "src/components/Notebook/ChartCell";
import { NotebookToolbar } from "src/components/Notebook/NotebookToolbar";
import { useStore } from "src/store";
import { useNotebook } from "src/api/notebook/queries";
import { CellType } from "src/components/Notebook/Cell/dto";
import { KEYBOARD_SHORTCUTS } from "src/utils/keyboard_shortcuts";
import { useHandleExecuteCell } from "src/components/Notebook/Cell";
import { useKeyboardShortcut } from "src/utils/keyboard_shortcuts/hooks";
import { Flex } from "src/components/design-system/Flex/Flex";
import { useCallback, useRef, useMemo } from "react";
import { useCreateCell, useDeleteCell, useReorderCell } from "src/api/cell";
import { useParams } from "react-router";

export function Notebook() {
  const params = useParams();
  const canvasId = params.id || "";
  const activeNotebookId = useStore((state) => state.canvas.activeNotebookId);
  const query = useNotebook(activeNotebookId);
  const handleExecuteCell = useHandleExecuteCell();
  const notebookRef = useRef<HTMLDivElement>(null);
  const createCellMutation = useCreateCell();
  const deleteCellMutation = useDeleteCell(
    activeNotebookId ?? undefined,
    canvasId || undefined
  );
  const reorderCellMutation = useReorderCell(canvasId || undefined);

  const handleExecute = useCallback(() => {
    const activeCellId = useStore.getState().notebook.activeCellId;
    handleExecuteCell(activeCellId);
  }, [handleExecuteCell]);

  const handleReorderCell = (
    movementType: "UP" | "DOWN" | "TOP" | "BOTTOM"
  ) => {
    const activeCellId = useStore.getState().notebook.activeCellId;
    if (activeCellId) {
      reorderCellMutation.mutate({ cellId: activeCellId, movementType });
    }
  };

  const handleDeleteCell = useCallback(() => {
    const activeCellId = useStore.getState().notebook.activeCellId;
    if (activeCellId) {
      deleteCellMutation.mutate(activeCellId, {
        onSuccess: () => {
          useStore.getState().notebook.setActiveCellId("");
        },
      });
    }
  }, [deleteCellMutation]);

  const handleCreateCell = useCallback(
    (cellType: CellType) => {
      const activeNotebookId = useStore.getState().canvas.activeNotebookId;
      createCellMutation.mutate({
        notebook_id: activeNotebookId,
        name: null,
        content: null,
        cell_type: cellType,
      });
    },
    [createCellMutation]
  );

  useKeyboardShortcut(
    handleExecute,
    KEYBOARD_SHORTCUTS.EXECUTE_NOTEBOOK_CELL,
    notebookRef
  );

  useKeyboardShortcut(
    () => handleReorderCell("UP"),
    KEYBOARD_SHORTCUTS.MOVE_CELL_UP,
    notebookRef
  );

  useKeyboardShortcut(
    () => handleReorderCell("TOP"),
    KEYBOARD_SHORTCUTS.MOVE_CELL_TOP,
    notebookRef
  );

  useKeyboardShortcut(
    () => handleReorderCell("DOWN"),
    KEYBOARD_SHORTCUTS.MOVE_CELL_DOWN,
    notebookRef
  );

  useKeyboardShortcut(
    () => handleReorderCell("BOTTOM"),
    KEYBOARD_SHORTCUTS.MOVE_CELL_BOTTOM,
    notebookRef
  );

  useKeyboardShortcut(
    () => handleCreateCell(CellType.MARKDOWN),
    KEYBOARD_SHORTCUTS.ADD_MARKDOWN_CELL
  );

  useKeyboardShortcut(
    () => handleCreateCell(CellType.SQL),
    KEYBOARD_SHORTCUTS.ADD_SQL_CELL
  );

  useKeyboardShortcut(
    () => handleCreateCell(CellType.CHART),
    KEYBOARD_SHORTCUTS.ADD_CHART_CELL
  );

  useKeyboardShortcut(
    handleDeleteCell,
    KEYBOARD_SHORTCUTS.DELETE_CELL,
    notebookRef
  );

  const cellComponents = useMemo(() => {
    if (!query.data?.cells) return null;
    return query.data.cells
      .sort((a, b) => {
        const orderA = a.display_order ?? Number.MAX_SAFE_INTEGER;
        const orderB = b.display_order ?? Number.MAX_SAFE_INTEGER;
        return orderA - orderB;
      })
      .map((cell) => {
        if (cell.cell_type === CellType.CHART) {
          return <ChartCell key={cell.id} cell_id={cell.id} />;
        }
        return <Cell key={cell.id} cell_id={cell.id} />;
      });
  }, [query.data?.cells]);

  if (query.isPending) {
    return <span>Loading...</span>;
  }

  if (query.isError) {
    return <span>Error...{query.error.message}</span>;
  }

  if (!query.data) {
    return <span>No data available</span>;
  }

  return (
    <Flex ref={notebookRef} direction="column" gap="md" flex="grow">
      <NotebookToolbar />
      <Flex direction="column" gap="md">
        {cellComponents}
      </Flex>
      <Flex height={"100px"} />
    </Flex>
  );
}
