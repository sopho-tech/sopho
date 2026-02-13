import { CellOutputState } from "src/components/Notebook/Cell";
import { useStore } from "src/store";
import styles from "src/components/Notebook/ChartCell/CellOutput/CellOutput.module.css";
import { BarChart, ChartType, LineChart, PieChart } from "src/components/Chart";
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
  const output = useStore((state) => state.cell.outputs[cellId]);
  const outputState = useStore((state) => state.cell.outputStates[cellId]);
  const chartContent = useStore((state) => state.cell.chartContents[cellId]);
  const chartType = getChartType(chartContent);

  console.log(output, chartContent);

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
          xAxisTickShow={barChartContent.x_axis_tick_show}
          yAxisTickShow={barChartContent.y_axis_tick_show}
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
          showDots={lineChartContent.show_dots !== "HIDE"}
          xAxisTickShow={lineChartContent.x_axis_tick_show}
          yAxisTickShow={lineChartContent.y_axis_tick_show}
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
