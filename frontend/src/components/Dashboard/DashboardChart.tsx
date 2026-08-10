import { useCell, useCellExecutionResult } from "src/api/cell";
import { useHandleExecuteCell } from "src/components/Notebook/Cell";
import {
  useExecutionPhase,
  ExecutionPhase,
  isAtLeast,
} from "src/components/Notebook/Cell/useExecutionPhase";
import { ChartSkeleton } from "src/components/Notebook/ExecutionIndicator";
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
  Box,
  Flex,
  Heading,
  Icon,
  IconButton,
} from "src/components/design-system";
import { validateMetricChartData } from "src/components/Chart/MetricChart/utils";
import { useStore, DashboardMode } from "src/store";
import { useAiConfiguration } from "src/api/ai_configuration";
import {
  useChartSummary,
  useGenerateChartSummary,
  useUpdateChartPrompt,
} from "src/api/ai_summary";
import { ChartSummary } from "src/components/Dashboard/ChartSummary";
import { SummaryPromptControl } from "src/components/Dashboard/SummaryPrompt";
import styles from "src/components/Dashboard/Dashboard.module.css";

const REFRESH_TOOLTIP = {
  text: "refresh chart",
  direction: "top",
} as const;

const STALE_OUTPUT_STYLE: React.CSSProperties = {
  opacity: 0.4,
  pointerEvents: "none",
  transition:
    "opacity var(--transition-duration-short) var(--transition-easing-standard-decelerate)",
};

type DashboardChartProps = {
  cellId: string;
  dashboardId: string;
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

function DashboardChartWithQuery({ cellId, dashboardId }: DashboardChartProps) {
  const cellQuery = useCell(cellId);
  const handleExecuteCell = useHandleExecuteCell();
  const { isRunning, phase } = useExecutionPhase(cellId);
  const { data: output } = useCellExecutionResult(cellId);
  const chartContent =
    cellQuery && cellQuery.data ? getChartContent(cellQuery.data) : null;
  const mode = useStore((state) => state.dashboard.mode);
  const getLayout = useStore((state) => state.dashboard.getLayout);
  const setLayout = useStore((state) => state.dashboard.setLayout);
  const isEditing = mode === DashboardMode.EDITING;
  const { data: aiConfiguration } = useAiConfiguration();
  const { data: summary } = useChartSummary(dashboardId, cellId);
  const { mutate: generateSummary, isPending } =
    useGenerateChartSummary(dashboardId);
  const { mutate: savePrompt, isPending: isSavingPrompt } = useUpdateChartPrompt(
    dashboardId,
    cellId
  );
  const isAiConfigured = aiConfiguration?.status === "live";
  const isSummarizing = summary?.status === "GENERATING" || isPending;

  const handleSummarize = useCallback(() => {
    if (isSummarizing) {
      return;
    }
    generateSummary(cellId);
  }, [cellId, isSummarizing, generateSummary]);

  useEffect(() => {
    handleExecuteCell(cellId);
  }, [cellId, handleExecuteCell]);

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
    handleExecuteCell(cellId);
  }, [cellId, handleExecuteCell]);

  const revealChrome = !isSummarizing && !isEditing && !isRunning;
  const showSkeleton = isLoading && isAtLeast(phase, ExecutionPhase.VISIBLE);

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
      revealChildrenOnHover
    >
      <Flex direction="row" justifyContent="space-between" gap="sm">
        <Flex direction="row" gap="xs" alignItems="center" overflow="hidden">
          <Heading
            accessbilityLevel={3}
            weight="semibold"
            size="sm"
            textColor="subtle"
          >
            {cellQuery?.data?.name}
          </Heading>
          {isAiConfigured && summary && (
            <Box display="flex" revealOnHover={revealChrome}>
              <ChartSummary summary={summary} />
            </Box>
          )}
        </Flex>
        <Flex
          direction="row"
          gap="xs"
          alignItems="center"
          revealOnHover={revealChrome}
        >
          {isAiConfigured && (
            <SummaryPromptControl
              iconType="sparkles"
              actionLabel="Summarise chart"
              dialogTitle="Chart summary prompt"
              dialogDescription="Shape how this chart is described. Leave it empty to use the default style."
              userPrompt={summary?.user_prompt ?? null}
              busy={isSummarizing}
              isSaving={isSavingPrompt}
              size="sm"
              onGenerate={handleSummarize}
              onSavePrompt={savePrompt}
            />
          )}
          <IconButton
            type="refresh"
            backgroundColor="default"
            iconColor="grey"
            onClick={handleRefresh}
            iconSize="md"
            busy={isRunning}
            busyAnimation="spin"
            tooltip={REFRESH_TOOLTIP}
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

      {showSkeleton && <ChartSkeleton />}

      {output && output.status === "error" && (
        <Flex
          alignItems="center"
          justifyContent="center"
          paddingX="lg"
          paddingY="lg"
          height="100%"
          aria-busy={isRunning}
          sx={isRunning ? STALE_OUTPUT_STYLE : undefined}
        >
          <BannerSlim
            type="error"
            message={`Check errors in ${cellQuery.data?.name}: ${output.error.message}.`}
          />
        </Flex>
      )}

      {!isLoading && output?.status === "success" && output.data && (
        <Box
          flex="grow"
          overflow="hidden"
          aria-busy={isRunning}
          sx={isRunning ? STALE_OUTPUT_STYLE : undefined}
        >
          <ChartRenderer chartContent={chartContent} output={output.data} />
        </Box>
      )}
    </Flex>
  );
}

export function DashboardChart({ cellId, dashboardId }: DashboardChartProps) {
  if (cellId.startsWith("_")) {
    return null;
  }
  return <DashboardChartWithQuery cellId={cellId} dashboardId={dashboardId} />;
}
