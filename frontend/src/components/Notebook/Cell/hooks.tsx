import { useCallback } from "react";
import { useExecuteCell, useExecuteCellPreview } from "src/api/cell";
import { CellType, ExecutionState } from "src/components/Notebook/Cell/dto";
import { getSelectedText } from "src/components/Notebook/CellEditor/editorRegistry";
import { useStore } from "src/store";

type ExecutionCallback = (() => void) | null | undefined;

function useStartCellExecution() {
  const setExecutionState = useStore((state) => state.cell.setExecutionState);

  return useCallback(
    (
      cellId: string,
      onSuccessCallback: ExecutionCallback,
      onErrorCallback: ExecutionCallback
    ) => {
      setExecutionState(cellId, ExecutionState.RUNNING);
      return {
        onSuccess: () => {
          setExecutionState(cellId, ExecutionState.COMPLETED);
          onSuccessCallback?.();
        },
        onError: () => {
          setExecutionState(cellId, ExecutionState.FAILED);
          onErrorCallback?.();
        },
      };
    },
    [setExecutionState]
  );
}

export function useHandleExecuteCell() {
  const { mutate } = useExecuteCell();
  const startCellExecution = useStartCellExecution();

  return useCallback(
    (
      cellId: string,
      onSuccessCallback: ExecutionCallback = null,
      onErrorCallback: ExecutionCallback = null
    ) => {
      mutate(
        cellId,
        startCellExecution(cellId, onSuccessCallback, onErrorCallback)
      );
    },
    [mutate, startCellExecution]
  );
}

export function useHandleExecuteCellPreview() {
  const { mutate } = useExecuteCellPreview();
  const startCellExecution = useStartCellExecution();

  return useCallback(
    (
      cellId: string,
      content: string,
      cellType: string,
      onSuccessCallback?: ExecutionCallback,
      onErrorCallback?: ExecutionCallback
    ) => {
      mutate(
        { cellId, content, cellType },
        startCellExecution(cellId, onSuccessCallback, onErrorCallback)
      );
    },
    [mutate, startCellExecution]
  );
}

export function useHandleRunCell() {
  const handleExecuteCell = useHandleExecuteCell();
  const handleExecuteCellPreview = useHandleExecuteCellPreview();

  return useCallback(
    (
      cellId: string,
      onSuccessCallback: ExecutionCallback = null,
      onErrorCallback: ExecutionCallback = null
    ) => {
      const selectedText = getSelectedText(cellId);
      if (!selectedText) {
        handleExecuteCell(cellId, onSuccessCallback, onErrorCallback);
        return;
      }

      handleExecuteCellPreview(
        cellId,
        selectedText,
        CellType.SQL,
        onSuccessCallback,
        onErrorCallback
      );
    },
    [handleExecuteCell, handleExecuteCellPreview]
  );
}
