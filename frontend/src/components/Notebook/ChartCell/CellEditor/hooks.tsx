import { useEffect, useState, useMemo } from "react";
import { useHandleExecuteCell } from "src/components/Notebook/Cell";
import { useStore } from "src/store";
import type { ChartContent } from "src/components/Notebook/Cell/dto";
import { useNotebook } from "src/api/notebook/queries";
import { CellOutputState, CellType } from "src/components/Notebook/Cell/dto";
import { ChartType } from "src/components/Chart";
import type { ExecuteCellResponseDto } from "src/components/Notebook/Cell/dto";
import { Icon, Flex, Text } from "src/components/design-system";
import { AggregateFunction } from "src/components/Notebook/dto";
import { getIconForDataType } from "src/utils/column_utils";

export function useSourceCellExecution(
  cellId: string,
  chartContent: ChartContent | null
) {
  const handleExecuteCell = useHandleExecuteCell();
  const [sourceCellId, setSourceCellId] = useState<string | null>(
    chartContent?.cell_id || null
  );
  const sourceCellOutput = useStore((state) =>
    sourceCellId ? state.cell.outputs[sourceCellId] : null
  );
  const setOutputState = useStore((state) => state.cell.setOutputState);

  const onSuccessCallback = () => {
    setOutputState(cellId, CellOutputState.PRESENT);
  };
  const onErrorCallback = () => {
    setOutputState(cellId, CellOutputState.ERROR);
  };
  const executeSourceCell = () => {
    if (sourceCellId) {
      setOutputState(cellId, CellOutputState.EXECUTING);
      handleExecuteCell(
        sourceCellId,
        false,
        onSuccessCallback,
        onErrorCallback
      );
    }
  };

  useEffect(() => {
    if (chartContent?.cell_id) {
      handleExecuteCell(chartContent?.cell_id, false);
      setSourceCellId(chartContent?.cell_id);
    }
  }, [chartContent?.cell_id]);

  useEffect(() => {
    if (sourceCellId) {
      handleExecuteCell(sourceCellId, false);
    }
  }, [sourceCellId]);

  return {
    sourceCellId,
    setSourceCellId,
    sourceCellOutput,
    executeSourceCell,
  };
}

export function useFormOptions(
  sourceCellOutput: ExecuteCellResponseDto | null | undefined
) {
  const activeNotebookId = useStore((state) => state.canvas.activeNotebookId);
  const notebookQuery = useNotebook(activeNotebookId);

  const cellOptions = useMemo(
    () =>
      notebookQuery.data?.cells
        ?.filter((cell) => cell.cell_type === CellType.SQL)
        ?.map((cell) => ({
          value: cell.id,
          label: cell.name || `Cell ${cell.display_order}`,
        })) || [],
    [notebookQuery.data?.cells]
  );

  const chartOptions = useMemo(
    () =>
      Object.entries(ChartType).map(([key, value]) => ({
        value,
        label: key.charAt(0) + key.slice(1).toLowerCase(),
      })),
    []
  );

  const yAxisAggregateFunctionsOptions = useMemo(
    () =>
      Object.entries(AggregateFunction).map(([key, value]) => ({
        value,
        label: key.charAt(0) + key.slice(1).toLowerCase(),
      })),
    []
  );

  const xAxisColumnOptions = useMemo(
    () =>
      sourceCellOutput?.columns && sourceCellOutput.columns.length > 0
        ? sourceCellOutput.columns.map((column) => ({
            value: column.column_name,
            label: (
              <Flex direction="row" gap="2xs" alignItems="center">
                <Icon
                  type={getIconForDataType(column.data_type)}
                  color="default"
                  strokeWidth={1.5}
                  size="sm"
                />
                <Text>{column.column_name}</Text>
              </Flex>
            ),
            textValue: column.column_name,
          }))
        : [],
    [sourceCellOutput?.columns]
  );

  const yAxisColumnOptions = useMemo(
    () =>
      sourceCellOutput?.columns && sourceCellOutput.columns.length > 0
        ? sourceCellOutput.columns.map((column) => ({
            value: column.column_name,
            label: (
              <Flex direction="row" gap="2xs" alignItems="center">
                <Icon
                  type={getIconForDataType(column.data_type)}
                  color="default"
                  strokeWidth={1.5}
                  size="sm"
                />
                <Text>{column.column_name}</Text>
              </Flex>
            ),
            textValue: column.column_name,
          }))
        : [],
    [sourceCellOutput?.columns]
  );

  return {
    cellOptions,
    chartOptions,
    xAxisColumnOptions,
    yAxisColumnOptions,
    yAxisAggregateFunctionsOptions,
  };
}
