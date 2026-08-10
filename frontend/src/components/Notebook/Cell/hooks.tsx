import { useCallback } from "react";
import { useExecuteCell, useExecuteCellPreview } from "src/api/cell";
import { ExecutionState } from "src/components/Notebook/Cell/dto";
import { useStore } from "src/store";

export function useHandleExecuteCell() {
  const { mutate } = useExecuteCell();
  const setExecutionState = useStore((state) => state.cell.setExecutionState);

  return useCallback(
    (
      cellId: string,
      onSuccessCallback: (() => void) | null = null,
      onErrorCallback: (() => void) | null = null
    ) => {
      setExecutionState(cellId, ExecutionState.RUNNING);
      mutate(cellId, {
        onSuccess: () => {
          setExecutionState(cellId, ExecutionState.COMPLETED);
          if (onSuccessCallback) {
            onSuccessCallback();
          }
        },
        onError: () => {
          setExecutionState(cellId, ExecutionState.FAILED);
          if (onErrorCallback) {
            onErrorCallback();
          }
        },
      });
    },
    [mutate, setExecutionState]
  );
}

export function useHandleExecuteCellPreview() {
  const { mutate } = useExecuteCellPreview();
  const setExecutionState = useStore((state) => state.cell.setExecutionState);

  return useCallback(
    (
      cellId: string,
      content: string,
      cellType: string,
      onSuccessCallback?: () => void,
      onErrorCallback?: () => void
    ) => {
      setExecutionState(cellId, ExecutionState.RUNNING);
      mutate(
        { cellId, content, cellType },
        {
          onSuccess: () => {
            setExecutionState(cellId, ExecutionState.COMPLETED);
            onSuccessCallback?.();
          },
          onError: () => {
            setExecutionState(cellId, ExecutionState.FAILED);
            onErrorCallback?.();
          },
        }
      );
    },
    [mutate, setExecutionState]
  );
}
