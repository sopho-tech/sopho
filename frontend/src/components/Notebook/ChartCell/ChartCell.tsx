import { useCallback } from "react";
import ChartCellStyles from "src/components/Notebook/ChartCell/ChartCell.module.css";
import activeCellStyles from "src/components/Notebook/activeCell.module.css";
import { ChartCellToolbar } from "src/components/Notebook/ChartCell/ChartCellToolbar/ChartCellToolbar";
import { CellOutput } from "src/components/Notebook/ChartCell/CellOutput/CellOutput";
import { CellEditor } from "src/components/Notebook/ChartCell/CellEditor";
import { useCellContainerClassName } from "src/components/Notebook/hooks";
import { useStore } from "src/store";

export function ChartCell({ cell_id }: { cell_id: string }) {
  const containerClassName = useCellContainerClassName(
    cell_id,
    ChartCellStyles.container,
    activeCellStyles.activeCell
  );
  const setActiveCellId = useStore((state) => state.notebook.setActiveCellId);

  const handleActivate = useCallback(() => {
    setActiveCellId(cell_id);
  }, [cell_id, setActiveCellId]);

  return (
    <div
      className={containerClassName}
      onFocus={handleActivate}
      onClick={handleActivate}
    >
      <ChartCellToolbar cellId={cell_id} />
      <div className={ChartCellStyles.editorAndOutputContainer}>
        <CellEditor cellId={cell_id} />
        <CellOutput cellId={cell_id} />
      </div>
    </div>
  );
}
