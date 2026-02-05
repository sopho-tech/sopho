import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebook/Cell";
import styles from "src/components/Notebook/ChartCell/CellOutput/CellOutput.module.css";
import { BarChart, ChartType, LineChart, PieChart } from "src/components/Chart";
import { useCell } from "src/api/cell/queries";
import { getChartContent } from "src/components/Notebook/Cell";
import cellStyles from "src/css/cell.module.css";
import {
  BarChartContent,
  getChartType,
  LineChartContent,
  PieChartContent,
} from "../../Cell/dto";

interface ChartCellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: ChartCellOutputProps) {
  const output = useCellOutputStore((state) => state.outputs[cellId]);
  const outputState = useCellOutputStore((state) => state.outputStates[cellId]);
  const cellQuery = useCell(cellId);
  const chartContent = cellQuery.data ? getChartContent(cellQuery.data) : null;
  const chartType = getChartType(chartContent);

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
    if (chartType === ChartType.BAR) {
      const barChartContent = chartContent as BarChartContent;
      return (
        <BarChart
          xAxis={barChartContent.x_axis}
          yAxis={barChartContent.y_axis}
          data={output.data}
          xAxisTitle={barChartContent.x_axis_title}
          yAxisTitle={barChartContent.y_axis_title}
        />
      );
    }
    if (chartType === ChartType.LINE) {
      const lineChartContent = chartContent as LineChartContent;
      return (
        <LineChart
          xAxis={lineChartContent.x_axis}
          yAxis={lineChartContent.y_axis}
          data={output.data}
          xAxisTitle={lineChartContent.x_axis_title}
          yAxisTitle={lineChartContent.y_axis_title}
        />
      );
    }
    if (chartType === ChartType.PIE) {
      const pieChartContent = chartContent as PieChartContent;
      return (
        <PieChart
          category={pieChartContent.category}
          value={pieChartContent.value}
          dimensions={output.columns?.map((column) => column.column_name) ?? []}
          data={output.data}
        />
      );
    }
  }

  return (
    <div className={`${styles.container} ${cellStyles.outputContainer}`}>
      {render()}
    </div>
  );
}
