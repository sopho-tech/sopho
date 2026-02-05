import { useCell, useExecuteCell } from "src/api/cell";
import { BarChart, ChartType, LineChart, PieChart } from "../Chart";
import { getChartContent } from "../Notebook/Cell";
import { useEffect, useState } from "react";
import {
  ExecuteCellResponseDto,
  BarChartContent,
  LineChartContent,
  PieChartContent,
  getChartType,
} from "../Notebook/Cell/dto";
import { Flex, Heading, Icon, IconButton } from "src/components/design-system";
import {
  useDashboardStore,
  DashboardMode,
} from "src/components/Dashboard/store";
import styles from "src/components/Dashboard/Dashboard.module.css";

type DashboardChartProps = {
  cellId: string;
};

function describeCellQuery(cellId: string) {
  if (cellId.startsWith("_")) {
    return null;
  }
  return useCell(cellId);
}

function ChartRenderer({
  chartContent,
  output,
}: {
  chartContent: BarChartContent | PieChartContent | LineChartContent;
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
        data={output.data as Object[]}
        xAxisTitle={barContent.x_axis_title}
        yAxisTitle={barContent.y_axis_title}
      />
    );
  }

  if (chartType === ChartType.LINE) {
    const lineContent = chartContent as LineChartContent;
    return (
      <LineChart
        xAxis={lineContent.x_axis}
        yAxis={lineContent.y_axis}
        data={output.data as Object[]}
        xAxisTitle={lineContent.x_axis_title}
        yAxisTitle={lineContent.y_axis_title}
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
        data={output.data as Object[]}
      />
    );
  }

  return null;
}

export function DashboardChart({ cellId }: DashboardChartProps) {
  const cellQuery = describeCellQuery(cellId);
  const executeCellMutation = useExecuteCell();
  const chartContent =
    cellQuery && cellQuery.data ? getChartContent(cellQuery.data) : null;
  const [output, setOutput] = useState<ExecuteCellResponseDto | null>(null);
  const { mode, getLayout, setLayout } = useDashboardStore();
  const isEditing = mode === DashboardMode.EDITING;

  useEffect(() => {
    if (cellId.startsWith("_")) {
      return;
    }
    executeCellMutation.mutate(cellId, {
      onSuccess: (data) => {
        setOutput(data);
      },
      onError: () => {},
    });
  }, []);

  const isLoading =
    cellQuery == null ||
    cellQuery.isLoading ||
    cellQuery.data == null ||
    chartContent == null ||
    output == null;

  const handleRemove = () => {
    const currentLayout = getLayout();
    const filteredLayout = currentLayout.filter((item) => item.i !== cellId);
    setLayout(filteredLayout);
  };

  const handleRefresh = () => {
    if (cellId.startsWith("_")) {
      return;
    }
    executeCellMutation.mutate(cellId, {
      onSuccess: (data) => {
        setOutput(data);
      },
      onError: () => {},
    });
  };

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

      {!isLoading && (
        <ChartRenderer chartContent={chartContent} output={output} />
      )}
    </Flex>
  );
}
