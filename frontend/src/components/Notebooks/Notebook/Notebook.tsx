import { useParams } from "react-router";
import { Cell } from "src/components/Notebooks/Notebook/Cell";
import { ChartCell } from "src/components/Notebooks/Notebook/ChartCell";
import { NotebookMenuBar } from "src/components/Notebooks/Notebook/NotebookMenuBar";
import { useNotebookStore } from "src/components/Notebooks/store";
import { useEffect, useRef } from "react";
import { useNotebook } from "src/api/notebook/queries";
import { CellType } from "src/components/Notebooks/Notebook/Cell/dto";
import NotebookStyles from "src/components/Notebooks/Notebook/Notebook.module.css";
import { KEYBOARD_SHORTCUTS } from "src/utils/keyboard_shortcuts";
import { useHandleExecuteCell } from "src/components/Notebooks/Notebook/Cell";
import { useKeyboardShortcut } from "src/utils/keyboard_shortcuts/hooks";

export function Notebook() {
  let params = useParams();
  const { currentNotebookId, setCurrentNotebookId, activeCellId } =
    useNotebookStore();
  const query = useNotebook(params.id!);
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

  useEffect(() => {
    if (currentNotebookId !== params.id) {
      setCurrentNotebookId(params.id!);
    }
  }, [params.id]);

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
    <div ref={notebookRef} className={NotebookStyles.container}>
      <div className={NotebookStyles.titleBar}>
        <div className={NotebookStyles.titleFirstRow}>
          <h3 className={NotebookStyles.titleBarNotebookName}>
            {query.data.name}
          </h3>
        </div>
        <div className={NotebookStyles.description}>
          <p>{query.data.description}</p>
        </div>
      </div>
      <div className={NotebookStyles.menubarContainer}>
        <NotebookMenuBar />
      </div>
      <div className={NotebookStyles.cells}>{generateCellComponents()}</div>
    </div>
  );
}
