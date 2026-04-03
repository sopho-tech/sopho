import { useEffect, useMemo } from "react";
import { Box, Flex, Text } from "src/components/design-system";
import {
  BarChart,
  LineChart,
  PieChart,
  MetricChart,
  ChartType,
} from "src/components/Chart";
import {
  type ChartRenderData,
  type RecommendedVisualizationData,
} from "src/components/ConversationalAnalytics/dto";
import {
  useExecuteQuery,
  useQueryExecutionResult,
} from "src/api/connection/queries";
import styles from "./QueryResultChart.module.css";

type QueryResultChartProps = {
  chartData: ChartRenderData;
};

export function QueryResultChart({ chartData }: QueryResultChartProps) {
  const { queryData, visualization } = chartData;
  const chartType = visualization.chart_type;

  const dimensions = useMemo(
    () => (queryData.columns ?? []).map((col) => col.column_name),
    [queryData.columns],
  );

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

  const data = queryData.data!;

  const renderChart = () => {
    switch (chartType) {
      case ChartType.BAR:
        return (
          <BarChart
            xAxis={visualization.x_axis ?? dimensions[0]}
            yAxis={visualization.y_axis ?? dimensions[1]}
            data={data}
          />
        );
      case ChartType.LINE:
        return (
          <LineChart
            xAxis={visualization.x_axis ?? dimensions[0]}
            yAxis={visualization.y_axis ?? dimensions[1]}
            data={data}
          />
        );
      case ChartType.PIE:
        return (
          <PieChart
            category={visualization.category ?? dimensions[0]}
            value={visualization.value ?? dimensions[1]}
            dimensions={dimensions}
            data={data}
          />
        );
      case ChartType.METRIC:
        return <MetricChart data={data} />;
      default:
        return null;
    }
  };

  return (
    <Box className={styles.container}>
      <Flex direction="column" gap="xs">
        <Box className={styles.reasoning}>
          <Text fontSize="xs" color="subtle">
            {visualization.reasoning}
          </Text>
        </Box>
        <Box className={styles.chartWrapper}>{renderChart()}</Box>
      </Flex>
    </Box>
  );
}

type QueryResultChartContainerProps = {
  connectionId: string;
  sql: string;
  visualization: RecommendedVisualizationData["visualization"];
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
