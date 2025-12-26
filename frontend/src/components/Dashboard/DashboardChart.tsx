import { useCell, useExecuteCell } from "src/api/cell";
import { BarChart } from "../Chart";
import { getChartContent } from "../Notebook/Cell";
import { useEffect, useState } from "react";
import { ExecuteCellResponseDto } from "../Notebook/Cell/dto";
import { Flex, Heading, Icon, IconButton } from "src/components/design-system";
import {
  useDashboardStore,
  DashboardMode,
} from "src/components/Dashboard/store";
import { useHandleExecuteCell } from "src/components/Notebook/Cell/hooks";
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

export function DashboardChart({ cellId }: DashboardChartProps) {
  const cellQuery = describeCellQuery(cellId);
  const executeCellMutation = useExecuteCell();
  const handleExecuteCell = useHandleExecuteCell();
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
    handleExecuteCell(cellId, true);
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
        <Heading accessbilityLevel={3} size="sm">
          {cellQuery?.data?.name}
        </Heading>
        <Flex direction="row" gap="2xs">
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
        <BarChart
          xAxis={chartContent.x_axis}
          yAxis={chartContent.y_axis}
          data={output.data as Object[]}
          orientation={chartContent.orientation}
          dimensions={output.columns?.map((column) => column.column_name) ?? []}
          sortOrder={chartContent.y_axis_sort_order}
          axisTickShow={chartContent.axis_tick_show}
          axisMinorTickShow={chartContent.axis_minor_tick_show}
        />
      )}
    </Flex>
  );
}
