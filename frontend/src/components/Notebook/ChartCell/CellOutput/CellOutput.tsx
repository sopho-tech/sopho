import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebook/Cell";
import styles from "src/components/Notebook/ChartCell/CellOutput/CellOutput.module.css";
import { BarChart } from "src/components/Chart";
import { useCell } from "src/api/cell/queries";
import { getChartContent } from "src/components/Notebook/Cell";
import cellStyles from "src/css/cell.module.css";

interface ChartCellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: ChartCellOutputProps) {
  const { getOutput, getOutputState } = useCellOutputStore();
  const outputState = getOutputState(cellId);
  const cellQuery = useCell(cellId);
  const chartContent = cellQuery.data ? getChartContent(cellQuery.data) : null;
  const output = cellId ? getOutput(cellId) : null;

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
        orientation={chartContent.orientation}
        dimensions={output.columns?.map((column) => column.column_name) ?? []}
        sortOrder={chartContent.y_axis_sort_order}
        axisTickShow={chartContent.axis_tick_show}
        axisMinorTickShow={chartContent.axis_minor_tick_show}
      />
    );
  }

  return (
    <div className={`${styles.container} ${cellStyles.outputContainer}`}>
      {render()}
    </div>
  );
}
