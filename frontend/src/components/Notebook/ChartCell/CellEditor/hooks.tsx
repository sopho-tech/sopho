import { useCallback, useEffect, useState, useMemo } from "react";
import { useHandleExecuteCell } from "src/components/Notebook/Cell";
import { useStore } from "src/store";
import type { ChartContent } from "src/components/Notebook/Cell/dto";
import { useNotebook } from "src/api/notebook/queries";
import { useCellExecutionResult } from "src/api/cell";
import { CellType } from "src/components/Notebook/Cell/dto";
import { ChartType } from "src/components/Chart";
import { Icon, Flex, Text } from "src/components/design-system";
import { AggregateFunction } from "src/components/Notebook/dto";
import { getIconForDataType } from "src/utils/column_utils";

export function useSourceCellExecution(
  _cellId: string,
  chartContent: ChartContent | null
) {
  const handleExecuteCell = useHandleExecuteCell();
  const [sourceCellId, setSourceCellId] = useState<string | null>(
    chartContent?.cell_id || null
  );

  const executeSourceCell = useCallback(() => {
    if (sourceCellId) {
      handleExecuteCell(sourceCellId);
    }
  }, [sourceCellId, handleExecuteCell]);

  useEffect(() => {
    if (chartContent?.cell_id) {
      setSourceCellId(chartContent.cell_id);
    }
  }, [chartContent?.cell_id]);

  useEffect(() => {
    if (sourceCellId) {
      handleExecuteCell(sourceCellId);
    }
  }, [sourceCellId, handleExecuteCell]);

  return {
    sourceCellId,
    setSourceCellId,
    executeSourceCell,
  };
}

export function useChartCellAutoLoad(
  cellId: string,
  initialChartContent: ChartContent | null
) {
  const handleExecuteCell = useHandleExecuteCell();
  const setChartContent = useStore((state) => state.cell.setChartContent);

  useEffect(() => {
    if (!initialChartContent) {
      return;
    }
    if (useStore.getState().cell.chartContents[cellId] == null) {
      setChartContent(cellId, initialChartContent);
    }
  }, [cellId, initialChartContent, setChartContent]);

  useEffect(() => {
    if (!initialChartContent?.cell_id) {
      return;
    }
    handleExecuteCell(cellId);
  }, [cellId, initialChartContent?.cell_id, handleExecuteCell]);
}

export function useFormOptions(sourceCellId: string | null) {
  const activeNotebookId = useStore((state) => state.canvas.activeNotebookId);
  const notebookQuery = useNotebook(activeNotebookId);
  const { data: sourceCellOutput } = useCellExecutionResult(sourceCellId ?? "");
  const columns =
    sourceCellOutput?.status === "success"
      ? sourceCellOutput.data.columns
      : null;

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
      columns && columns.length > 0
        ? columns.map((column) => ({
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
    [columns]
  );

  const yAxisColumnOptions = useMemo(
    () =>
      columns && columns.length > 0
        ? columns.map((column) => ({
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
    [columns]
  );

  return {
    cellOptions,
    chartOptions,
    xAxisColumnOptions,
    yAxisColumnOptions,
    yAxisAggregateFunctionsOptions,
    isAwaitingSourceColumns: sourceCellId != null && sourceCellOutput == null,
  };
}
