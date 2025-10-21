import { useParams } from "react-router";
import { NewAssetButton } from "src/components/NewAssetButton";
import { Cell } from "src/components/Notebooks/Notebook/Cell";
import { ChartCell } from "src/components/Notebooks/Notebook/ChartCell";
import { NotebookMenuBar } from "src/components/Notebooks/Notebook/NotebookMenuBar";
import { useNotebookStore } from "src/components/Notebooks/store";
import { useEffect } from "react";
import { useNotebook, useUpdateNotebook } from "src/api/notebook/queries";
import { CellType } from "src/components/Notebooks/Notebook/Cell/dto";
import NotebookStyles from "src/components/Notebooks/Notebook/Notebook.module.css";

export function Notebook() {
  let params = useParams();
  const { currentNotebookId, setCurrentNotebookId } = useNotebookStore();
  const query = useNotebook(params.id!);
  const updateNotebookMutation = useUpdateNotebook();

  useEffect(() => {
    if (currentNotebookId !== params.id) {
      setCurrentNotebookId(params.id!);
    }
  }, [params.id]);

  function saveNotebook() {
    if (!query.data) return;
    updateNotebookMutation.mutate({
      notebookId: currentNotebookId,
      payload: query.data,
    });
  }

  if (query.isPending) {
    return <span>Loading...</span>;
  }

  if (query.isError) {
    return <span>Error...{query.error.message}</span>;
  }

  if (!query.data) {
    return <span>No data available</span>;
  }

  const cellComponents = query.data.cells
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

  return (
    <div className={`${NotebookStyles.container}`}>
      <div className={NotebookStyles.titleBar}>
        <div className={NotebookStyles.titleFirstRow}>
          <h1 className={NotebookStyles.titleBarNotebookName}>
            {query.data.name}
          </h1>
          <NewAssetButton
            buttonText="Save"
            className={NotebookStyles.saveButton}
            onClick={saveNotebook}
          />
        </div>
        <div className={NotebookStyles.description}>
          <p>{query.data.description} ??</p>
        </div>
      </div>
      <NotebookMenuBar />
      <div className={NotebookStyles.cells}>{cellComponents}</div>
    </div>
  );
}
