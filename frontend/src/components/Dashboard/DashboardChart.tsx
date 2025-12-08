import { useCell, useExecuteCell } from "src/api/cell";
import { BarChart } from "../Chart";
import { getChartContent } from "../Notebook/Cell";
import { useEffect, useState } from "react";
import { ExecuteCellResponseDto } from "../Notebook/Cell/dto";
import { Flex, Heading } from "src/components/design-system";

type DashboardChartProps = {
  cellId: string;
};

export function DashboardChart({ cellId }: DashboardChartProps) {
  const cellQuery = useCell(cellId);
  const executeCellMutation = useExecuteCell();
  const chartContent = cellQuery.data ? getChartContent(cellQuery.data) : null;
  const [output, setOutput] = useState<ExecuteCellResponseDto | null>(null);

  useEffect(() => {
    executeCellMutation.mutate(cellId, {
      onSuccess: (data) => {
        setOutput(data);
      },
      onError: () => {},
    });
  }, []);

  const isLoading =
    cellQuery.isLoading ||
    cellQuery.data == null ||
    chartContent == null ||
    output == null;

  return (
    <Flex
      height="100%"
      borderRadius="lg"
      border="default"
      shadow="2xs"
      color="white"
      direction="column"
      paddingX="md"
      paddingY="md"
      gap="sm"
      overflow="hidden"
    >
      <Flex>
        <Heading accessbilityLevel={3} size="sm">
          {cellQuery.data?.name}
        </Heading>
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
