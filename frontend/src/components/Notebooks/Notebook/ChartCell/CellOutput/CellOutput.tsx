import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebooks/Notebook/Cell";
import styles from "src/components/Notebooks/Notebook/ChartCell/CellOutput/CellOutput.module.css";
import { BarChart } from "src/components/Chart";
import { useCell } from "src/api/cell/queries";
import { getChartContent } from "src/components/Notebooks/Notebook/Cell";

interface ChartCellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: ChartCellOutputProps) {
  const { getOutput, getOutputState } = useCellOutputStore();
  const outputState = getOutputState(cellId);
  const cellQuery = useCell(cellId);
  const chartContent = cellQuery.data ? getChartContent(cellQuery.data) : null;
  const sourceCellId = chartContent?.cell_id || null;
  const output = sourceCellId ? getOutput(sourceCellId) : null;

  function render() {
    if (
      outputState === undefined ||
      outputState === CellOutputState.ABSENT ||
      output === null ||
      output === undefined ||
      chartContent === null ||
      output.data === null ||
      output.data === undefined
    ) {
      return null;
    }
    return (
      <BarChart
        xAxis={chartContent.x_axis}
        yAxis={chartContent.y_axis}
        data={output.data}
      />
    );
  }

  return <div className={styles.container}>{render()}</div>;
}
