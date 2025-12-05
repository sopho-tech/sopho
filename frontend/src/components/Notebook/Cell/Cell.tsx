import CellStyles from "src/components/Notebook/Cell/Cell.module.css";
import { CellEditor } from "src/components/Notebook/CellEditor/CellEditor";
import { CellToolbar } from "src/components/Notebook/CellToolbar/CellToolbar";
import { CellOutput } from "src/components/Notebook/CellOutput/CellOutput";

export function Cell({ cell_id }: { cell_id: string }) {
  return (
    <div className={CellStyles.cellContainer}>
      <div>
        <CellToolbar cellId={cell_id} />
        <CellEditor cellId={cell_id} />
      </div>
      <CellOutput cellId={cell_id} />
    </div>
  );
}
