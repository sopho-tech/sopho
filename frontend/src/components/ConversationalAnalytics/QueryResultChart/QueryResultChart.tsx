import { useEffect, useMemo } from "react";
import { Box, Flex, Text } from "src/components/design-system";
import {
  BarChart,
  LineChart,
  PieChart,
  MetricChart,
  ChartType,
  type ChartSeriesSpec,
} from "src/components/Chart";
import {
  type ChartRenderData,
  type Visualization,
  type VisualizationSeries,
} from "src/components/ConversationalAnalytics/dto";
import {
  useExecuteQuery,
  useQueryExecutionResult,
} from "src/api/connection/queries";
import styles from "./QueryResultChart.module.css";

type QueryResultChartProps = {
  chartData: ChartRenderData;
};

function toSeriesSpecs(series: VisualizationSeries[]): ChartSeriesSpec[] {
  return series.map((entry) => ({
    dataKey: entry.data_key,
    name: entry.name,
    colorIndex: entry.color_index,
  }));
}

function renderChart(visualization: Visualization, data: object[]) {
  switch (visualization.chart_type) {
    case ChartType.BAR:
      return (
        <BarChart
          xAxis={visualization.x_axis}
          series={toSeriesSpecs(visualization.series)}
          data={data}
        />
      );
    case ChartType.LINE:
      return (
        <LineChart
          xAxis={visualization.x_axis}
          series={toSeriesSpecs(visualization.series)}
          data={data}
        />
      );
    case ChartType.PIE:
      return (
        <PieChart
          category={visualization.category}
          value={visualization.value}
          data={data}
        />
      );
    case ChartType.METRIC:
      return <MetricChart data={data} />;
  }
}

export function QueryResultChart({ chartData }: QueryResultChartProps) {
  const { queryData, visualization } = chartData;

  if (!queryData.data?.length) {
    return (
      <Flex
        alignItems="center"
        justifyContent="center"
        paddingY="lg"
        paddingX="lg"
      >
        <Text fontSize="sm" color="subtle">
          Query returned no results
        </Text>
      </Flex>
    );
  }

  return (
    <Box className={styles.container}>
      <Box className={styles.chartWrapper}>
        {renderChart(visualization, queryData.data)}
      </Box>
    </Box>
  );
}

type QueryResultChartContainerProps = {
  connectionId: string;
  sql: string;
  visualization: Visualization;
};

export function QueryResultChartContainer({
  connectionId,
  sql,
  visualization,
}: QueryResultChartContainerProps) {
  const executeQuery = useExecuteQuery();
  const { data: executionResult } = useQueryExecutionResult(connectionId, sql);

  useEffect(() => {
    if (!sql) return;
    executeQuery.mutate({ connectionId, query: sql });
  }, [sql]);

  const chartData = useMemo<ChartRenderData | null>(() => {
    if (!executionResult || executionResult.status !== "success") return null;
    return { queryData: executionResult.data, visualization };
  }, [visualization, executionResult]);

  if (!chartData) return null;

  return <QueryResultChart chartData={chartData} />;
}
