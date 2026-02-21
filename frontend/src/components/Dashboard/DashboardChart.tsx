import { useCell, useExecuteCell, useCellExecutionResult } from "src/api/cell";
import {
  BarChart,
  ChartType,
  LineChart,
  MetricChart,
  PieChart,
} from "../Chart";
import { getChartContent } from "../Notebook/Cell";
import { useCallback, useEffect } from "react";
import {
  ExecuteCellResponseDto,
  BarChartContent,
  LineChartContent,
  MetricChartContent,
  PieChartContent,
  getChartType,
} from "../Notebook/Cell/dto";
import {
  BannerSlim,
  Flex,
  Heading,
  Icon,
  IconButton,
} from "src/components/design-system";
import { validateMetricChartData } from "src/components/Chart/MetricChart/utils";
import { useStore, DashboardMode } from "src/store";
import styles from "src/components/Dashboard/Dashboard.module.css";

type DashboardChartProps = {
  cellId: string;
};

function ChartRenderer({
  chartContent,
  output,
}: {
  chartContent:
    | BarChartContent
    | PieChartContent
    | LineChartContent
    | MetricChartContent;
  output: ExecuteCellResponseDto;
}) {
  const chartType = getChartType(chartContent);
  const dimensions = output.columns?.map((column) => column.column_name) ?? [];

  if (chartType === ChartType.BAR) {
    const barContent = chartContent as BarChartContent;
    return (
      <BarChart
        xAxis={barContent.x_axis}
        yAxis={barContent.y_axis}
        data={output.data as object[]}
        xAxisTitle={barContent.x_axis_title}
        yAxisTitle={barContent.y_axis_title}
        xAxisTickShow={barContent.x_axis_tick_show}
        yAxisTickShow={barContent.y_axis_tick_show}
      />
    );
  }

  if (chartType === ChartType.LINE) {
    const lineContent = chartContent as LineChartContent;
    return (
      <LineChart
        xAxis={lineContent.x_axis}
        yAxis={lineContent.y_axis}
        data={output.data as object[]}
        xAxisTitle={lineContent.x_axis_title}
        yAxisTitle={lineContent.y_axis_title}
        showDots={lineContent.show_dots !== "HIDE"}
        xAxisTickShow={lineContent.x_axis_tick_show}
        yAxisTickShow={lineContent.y_axis_tick_show}
      />
    );
  }

  if (chartType === ChartType.PIE) {
    const pieContent = chartContent as PieChartContent;
    return (
      <PieChart
        category={pieContent.category}
        value={pieContent.value}
        dimensions={dimensions}
        data={output.data as object[]}
      />
    );
  }

  if (chartType === ChartType.METRIC) {
    const metricChartContent = chartContent as MetricChartContent;
    const chartData = (output.data ?? []) as object[];
    const metricError = validateMetricChartData(chartData);
    if (metricError) {
      return (
        <Flex
          alignItems="center"
          justifyContent="center"
          paddingX="lg"
          paddingY="lg"
          height="100%"
        >
          <BannerSlim type="error" message={metricError} />
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

  return null;
}

function DashboardChartWithQuery({ cellId }: DashboardChartProps) {
  const cellQuery = useCell(cellId);
  const executeCellMutation = useExecuteCell();
  const { data: output } = useCellExecutionResult(cellId);
  const chartContent =
    cellQuery && cellQuery.data ? getChartContent(cellQuery.data) : null;
  const mode = useStore((state) => state.dashboard.mode);
  const getLayout = useStore((state) => state.dashboard.getLayout);
  const setLayout = useStore((state) => state.dashboard.setLayout);
  const isEditing = mode === DashboardMode.EDITING;

  useEffect(() => {
    executeCellMutation.mutate(cellId);
  }, [cellId, executeCellMutation.mutate]);

  const isLoading =
    cellQuery == null ||
    cellQuery.isLoading ||
    cellQuery.data == null ||
    chartContent == null ||
    output == null;

  const handleRemove = useCallback(() => {
    const currentLayout = getLayout();
    const filteredLayout = currentLayout.filter((item) => item.i !== cellId);
    setLayout(filteredLayout);
  }, [cellId, getLayout, setLayout]);

  const handleRefresh = useCallback(() => {
    executeCellMutation.mutate(cellId);
  }, [cellId, executeCellMutation.mutate]);

  return (
    <Flex
      height="100%"
      borderRadius="lg"
      border="default"
      shadow="2xs"
      backgroundColor="white"
      direction="column"
      paddingX="md"
      paddingY="md"
      gap="sm"
      overflow="hidden"
    >
      <Flex direction="row" justifyContent="space-between">
        <Heading
          accessbilityLevel={3}
          weight="semibold"
          size="sm"
          textColor="subtle"
        >
          {cellQuery?.data?.name}
        </Heading>
        <Flex direction="row" gap="xs">
          <IconButton
            type="refresh"
            backgroundColor="default"
            iconColor="grey"
            onClick={handleRefresh}
            iconSize="md"
          />
          {isEditing && (
            <>
              <IconButton
                type="delete"
                backgroundColor="default"
                iconColor="grey"
                onClick={handleRemove}
                iconSize="md"
              />
              <div className={styles.dragHandle}>
                <Icon type="grip_vertical" color="grey" size="md" />
              </div>
            </>
          )}
        </Flex>
      </Flex>

      {output && output.status === "error" && (
        <Flex
          alignItems="center"
          justifyContent="center"
          paddingX="lg"
          paddingY="lg"
          height="100%"
        >
          <BannerSlim
            type="error"
            message={`Check errors in ${cellQuery.data?.name}: ${output.error.message}.`}
          />
        </Flex>
      )}

      {!isLoading && output?.status === "success" && output.data && (
        <ChartRenderer chartContent={chartContent} output={output.data} />
      )}
    </Flex>
  );
}

export function DashboardChart({ cellId }: DashboardChartProps) {
  if (cellId.startsWith("_")) {
    return null;
  }
  return <DashboardChartWithQuery cellId={cellId} />;
}
