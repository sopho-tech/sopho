import { useExecuteCell } from "src/api/cell";
import { useStore } from "src/store";
import {
  ExecutionState,
  CellOutputState,
} from "src/components/Notebook/Cell/dto";

export function useHandleExecuteChartCell() {
  const setOutput = useStore((state) => state.chartCell.setOutput);
  const setExecutionState = useStore((state) => state.chartCell.setExecutionState);
  const setOutputState = useStore((state) => state.chartCell.setOutputState);
  const executeCellMutation = useExecuteCell();

  return (cellId: string) => {
    setExecutionState(cellId, ExecutionState.RUNNING);
    executeCellMutation.mutate(cellId, {
      onSuccess: (data) => {
        setOutput(cellId, data);
        setExecutionState(cellId, ExecutionState.COMPLETED);
        if (data != null) {
          setOutputState(cellId, CellOutputState.PRESENT);
        }
      },
      onError: () => {
        setExecutionState(cellId, ExecutionState.FAILED);
      },
    });
  };
}
