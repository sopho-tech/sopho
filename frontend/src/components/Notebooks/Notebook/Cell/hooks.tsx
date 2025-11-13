import { useExecuteCell } from "src/api/cell";
import { useCellOutputStore } from "src/components/Notebooks/Notebook/Cell/store";
import {
  ExecutionState,
  CellOutputState,
} from "src/components/Notebooks/Notebook/Cell/dto";

export function useHandleExecuteCell() {
  const { setOutput, setExecutionState, setOutputState } = useCellOutputStore();
  const executeCellMutation = useExecuteCell();

  return (
    cellId: string,
    shouldSetOutputState: boolean = true,
    onSuccessCallback: (() => void) | null = null,
    onErrorCallback: (() => void) | null = null
  ) => {
    setExecutionState(cellId, ExecutionState.RUNNING);
    executeCellMutation.mutate(cellId, {
      onSuccess: (data) => {
        setOutput(cellId, data);
        setExecutionState(cellId, ExecutionState.COMPLETED);
        if (data != null && shouldSetOutputState) {
          setOutputState(cellId, CellOutputState.PRESENT);
        }
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
  };
}
