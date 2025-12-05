import ChartCellStyles from "src/components/Notebook/ChartCell/ChartCell.module.css";
import { ChartCellToolbar } from "src/components/Notebook/ChartCell/ChartCellToolbar/ChartCellToolbar";
import { CellOutput } from "src/components/Notebook/ChartCell/CellOutput/CellOutput";
import { CellEditor } from "src/components/Notebook/ChartCell/CellEditor";

export function ChartCell({ cell_id }: { cell_id: string }) {
  return (
    <div className={ChartCellStyles.container}>
      <ChartCellToolbar cellId={cell_id} />
      <div className={ChartCellStyles.editorAndOutputContainer}>
        <CellEditor cellId={cell_id} />
        <CellOutput cellId={cell_id} />
      </div>
    </div>
  );
}
