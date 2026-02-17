import { useMemo } from "react";
import { useCell, useCellExecutionResult } from "src/api/cell";
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
import {
  BannerSlim,
  Flex,
  Heading,
  Icon,
  Text,
} from "src/components/design-system";

interface ChartCellOutputProps {
  cellId: string;
}

export function CellOutput({ cellId }: ChartCellOutputProps) {
  const { data: output } = useCellExecutionResult(cellId);
  const chartContent = useStore((state) => state.cell.chartContents[cellId]);
  const chartType = getChartType(chartContent);
  const cellQuery = useCell(chartContent?.cell_id ?? "");

  const executionData = output?.status === "success" ? output.data : null;
  const pieDimensions = useMemo(
    () =>
      executionData?.columns?.map(
        (col: { column_name: string }) => col.column_name
      ) ?? [],
    [executionData?.columns]
  );

  function render() {
    if (output && output.status === "error") {
      const message = `Check errors in ${cellQuery.data?.name}: ${output.error.message}. `;
      return (
        <Flex
          alignItems="center"
          justifyContent="center"
          paddingX="lg"
          paddingY="lg"
          height="100%"
        >
          <BannerSlim type="error" message={message} />
        </Flex>
      );
    }

    if (
      output == null ||
      chartContent == null ||
      output.status !== "success" ||
      executionData == null
    ) {
      return (
        <Flex
          as="section"
          direction="column"
          gap="md"
          alignItems="center"
          alignSelf="center"
          paddingY="lg"
          paddingX="lg"
        >
          <Icon
            type="bar_chart"
            color="lightgrey"
            size="2xl"
            interactive={false}
          />
          <Flex
            direction="column"
            gap="xs"
            alignItems="center"
            sx={{ textAlign: "center" }}
          >
            <Heading accessbilityLevel={2} textColor="default">
              No chart rendered
            </Heading>
            <Text as="p" color="subtle">
              Press on the run button to render the chart
            </Text>
          </Flex>
        </Flex>
      );
    }
    const chartData = executionData.data ?? [];
    if (chartType === ChartType.BAR) {
      const barChartContent = chartContent as BarChartContent;
      return (
        <BarChart
          xAxis={barChartContent.x_axis}
          yAxis={barChartContent.y_axis}
          data={chartData}
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
          data={chartData}
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
          dimensions={pieDimensions}
          data={chartData}
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
