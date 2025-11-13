import { useEffect, useState, useMemo } from "react";
import { useHandleExecuteCell } from "src/components/Notebooks/Notebook/Cell";
import { useCellOutputStore } from "src/components/Notebooks/Notebook/Cell/store";
import type { ChartContent } from "src/components/Notebooks/Notebook/Cell/dto";
import { useNotebook } from "src/api/notebook/queries";
import {
  CellOutputState,
  CellType,
} from "src/components/Notebooks/Notebook/Cell/dto";
import { ChartType } from "src/components/Chart";
import type { ExecuteCellResponseDto } from "src/components/Notebooks/Notebook/Cell/dto";
import { useNotebookStore } from "src/components/Notebooks/store";

export function useSourceCellExecution(
  cellId: string,
  chartContent: ChartContent | null
) {
  const handleExecuteCell = useHandleExecuteCell();
  const { getOutput, setOutputState, getOutputState } = useCellOutputStore();
  const [sourceCellId, setSourceCellId] = useState<string | null>(
    chartContent?.cell_id || null
  );
  const sourceCellOutput = sourceCellId ? getOutput(sourceCellId) : null;

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
  const { currentNotebookId } = useNotebookStore();
  const notebookQuery = useNotebook(currentNotebookId);

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

  const columnOptions = useMemo(
    () =>
      sourceCellOutput?.column_names && sourceCellOutput.column_names.length > 0
        ? sourceCellOutput.column_names.map((column) => ({
            value: column,
            label: column,
          }))
        : [],
    [sourceCellOutput?.column_names]
  );

  return {
    cellOptions,
    chartOptions,
    columnOptions,
  };
}
