import { Cell } from "src/components/Notebook/Cell";
import { ChartCell } from "src/components/Notebook/ChartCell";
import { NotebookMenuBar } from "src/components/Notebook/NotebookMenuBar";
import { useNotebookStore } from "src/components/Notebook/store";
import { useNotebook } from "src/api/notebook/queries";
import { CellType } from "src/components/Notebook/Cell/dto";
import { KEYBOARD_SHORTCUTS } from "src/utils/keyboard_shortcuts";
import { useHandleExecuteCell } from "src/components/Notebook/Cell";
import { useKeyboardShortcut } from "src/utils/keyboard_shortcuts/hooks";
import { Flex } from "src/components/design-system/Flex/Flex";
import { Sticky } from "src/components/design-system/Sticky/Sticky";
import { useRef } from "react";
import { useCanvasStore } from "src/components/Canvases/store";

export function Notebook() {
  const { activeCellId } = useNotebookStore();
  const { activeNotebookId } = useCanvasStore();
  const query = useNotebook(activeNotebookId);
  const handleExecuteCell = useHandleExecuteCell();
  const notebookRef = useRef<HTMLDivElement>(null);

  function handleExecute() {
    handleExecuteCell(activeCellId);
  }

  useKeyboardShortcut(
    notebookRef,
    handleExecute,
    KEYBOARD_SHORTCUTS.EXECUTE_NOTEBOOK_CELL
  );

  if (query.isPending) {
    return <span>Loading...</span>;
  }

  if (query.isError) {
    return <span>Error...{query.error.message}</span>;
  }

  if (!query.data) {
    return <span>No data available</span>;
  }

  function generateCellComponents() {
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
  }

  return (
    <Flex ref={notebookRef} direction="column" gap="md" flex="grow">
      <Sticky top={0} zIndex={"10"}>
        <NotebookMenuBar />
      </Sticky>
      <Flex direction="column" gap="md">
        {generateCellComponents()}
      </Flex>
    </Flex>
  );
}
