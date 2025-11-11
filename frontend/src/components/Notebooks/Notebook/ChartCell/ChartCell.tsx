import ChartCellStyles from "src/components/Notebooks/Notebook/ChartCell/ChartCell.module.css";
import { ChartCellToolbar } from "src/components/Notebooks/Notebook/ChartCell/ChartCellToolbar/ChartCellToolbar";
import { ChartCellOutput } from "src/components/Notebooks/Notebook/ChartCell/ChartCellOutput/ChartCellOutput";
import { CellEditor } from "src/components/Notebooks/Notebook/ChartCell/CellEditor";

export function ChartCell({ cell_id }: { cell_id: string }) {
  return (
    <div className={ChartCellStyles.chartCellContainer}>
      <div>
        <ChartCellToolbar cellId={cell_id} />
        <CellEditor cellId={cell_id} />
      </div>
      <ChartCellOutput cellId={cell_id} />
    </div>
  );
}
