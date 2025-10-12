import ChartCellStyles from "src/components/Notebooks/Notebook/ChartCell/ChartCell.module.css";
import { ChartCellEditor } from "src/components/Notebooks/Notebook/ChartCell/ChartCellEditor/ChartCellEditor";
import { ChartCellToolbar } from "src/components/Notebooks/Notebook/ChartCell/ChartCellToolbar/ChartCellToolbar";
import { ChartCellOutput } from "src/components/Notebooks/Notebook/ChartCell/ChartCellOutput/ChartCellOutput";

export function ChartCell({ cell_id }: { cell_id: string }) {
  return (
    <div className={ChartCellStyles.chartCellContainer}>
      <div>
        <ChartCellToolbar cellId={cell_id} />
        <ChartCellEditor cellId={cell_id} />
      </div>
      <ChartCellOutput cellId={cell_id} />
    </div>
  );
}
