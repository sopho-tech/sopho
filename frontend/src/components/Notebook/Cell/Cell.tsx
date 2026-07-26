import { useCallback } from "react";
import styles from "src/components/Notebook/Cell/Cell.module.css";
import activeCellStyles from "src/components/Notebook/activeCell.module.css";
import { CellEditor } from "src/components/Notebook/CellEditor/CellEditor";
import { CellToolbar } from "src/components/Notebook/CellToolbar/CellToolbar";
import { CellOutput } from "src/components/Notebook/CellOutput/CellOutput";
import { useCellContainerClassName } from "src/components/Notebook/hooks";
import { useStore } from "src/store";

export function Cell({ cell_id }: { cell_id: string }) {
  const styleClassName = useCellContainerClassName(
    cell_id,
    styles.cellContainer,
    activeCellStyles.activeCell
  );
  const setActiveCellId = useStore((state) => state.notebook.setActiveCellId);
  const handleActivate = useCallback(() => {
    setActiveCellId(cell_id);
  }, [cell_id, setActiveCellId]);

  return (
    <div
      id={cell_id}
      className={styleClassName}
      onFocus={handleActivate}
      onClick={handleActivate}
    >
      <div>
        <CellToolbar cellId={cell_id} />
        <CellEditor cellId={cell_id} />
      </div>
      <CellOutput cellId={cell_id} />
    </div>
  );
}
