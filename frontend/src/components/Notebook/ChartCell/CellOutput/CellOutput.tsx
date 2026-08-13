import { useCell, useCellExecutionResult } from "src/api/cell";
import { useStore } from "src/store";
import styles from "src/components/Notebook/ChartCell/CellOutput/CellOutput.module.css";
import {
  BarChart,
  ChartType,
  LineChart,
  MetricChart,
  PieChart,
  findMissingSeriesColumns,
  toAxisChartProps,
} from "src/components/Chart";
import cellStyles from "src/css/cell.module.css";
import {
  BarChartContent,
  getChartType,
  LineChartContent,
  MetricChartContent,
  PieChartContent,
} from "../../Cell/dto";
import {
  BannerSlim,
  Flex,
  Heading,
  Icon,
  Text,
} from "src/components/design-system";
import { validateMetricChartData } from "src/components/Chart/MetricChart/utils";
import {
  useExecutionPhase,
  ExecutionPhase,
  isAtLeast,
} from "src/components/Notebook/Cell/useExecutionPhase";
import { ChartSkeleton } from "src/components/Notebook/ExecutionIndicator";

interface ChartCellOutputProps {
  cellId: string;
}

const STALE_OUTPUT_STYLE: React.CSSProperties = {
  opacity: 0.4,
  pointerEvents: "none",
  transition:
    "opacity var(--transition-duration-short) var(--transition-easing-standard-decelerate)",
};

export function CellOutput({ cellId }: ChartCellOutputProps) {
  const { data: output } = useCellExecutionResult(cellId);
  const { isRunning, phase } = useExecutionPhase(cellId);
  const chartContent = useStore((state) => state.cell.chartContents[cellId]);
  const chartType = getChartType(chartContent);
  const cellQuery = useCell(chartContent?.cell_id ?? "");

  const executionData = output?.status === "success" ? output.data : null;

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
      if (isAtLeast(phase, ExecutionPhase.VISIBLE)) {
        return <ChartSkeleton />;
      }

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
    if (chartType === ChartType.BAR || chartType === ChartType.LINE) {
      const axisContent = chartContent as BarChartContent | LineChartContent;
      const axisProps = toAxisChartProps(axisContent);
      const missing = findMissingSeriesColumns(
        axisProps.series,
        executionData.columns?.map((column) => column.column_name) ?? []
      );
      if (missing.length > 0) {
        return (
          <Flex
            alignItems="center"
            justifyContent="center"
            paddingX="lg"
            paddingY="lg"
            height="100%"
          >
            <BannerSlim
              type="error"
              message={`These series are not in the query result: ${missing.join(", ")}. Re-run the source cell or update the chart.`}
            />
          </Flex>
        );
      }
      return chartType === ChartType.BAR ? (
        <BarChart
          {...axisProps}
          data={chartData}
          stacked={axisContent.bar_layout === "STACKED"}
        />
      ) : (
        <LineChart
          {...axisProps}
          data={chartData}
          showDots={(axisContent as LineChartContent).show_dots !== "HIDE"}
        />
      );
    }
    if (chartType === ChartType.PIE) {
      const pieChartContent = chartContent as PieChartContent;
      return (
        <PieChart
          category={pieChartContent.category}
          value={pieChartContent.value}
          data={chartData}
        />
      );
    }
    if (chartType === ChartType.METRIC) {
      const metricChartContent = chartContent as MetricChartContent;
      const metricError = validateMetricChartData(chartData);
      if (metricError) {
        const message = metricError;
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
      return (
        <MetricChart
          data={chartData}
          precision={metricChartContent.decimal_precision}
          suffix={metricChartContent.suffix}
          format={metricChartContent.format}
        />
      );
    }
  }

  const hasRenderedOutput = output != null;

  return (
    <div
      className={`${styles.container} ${cellStyles.outputContainer}`}
      style={isRunning && hasRenderedOutput ? STALE_OUTPUT_STYLE : undefined}
      aria-busy={isRunning}
    >
      {render()}
    </div>
  );
}
