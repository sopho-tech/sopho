import { useExecuteCell } from "src/api/cell";
import { useChartCellExecutionStore } from "src/components/Notebook/ChartCell/store";
import {
  ExecutionState,
  CellOutputState,
} from "src/components/Notebook/Cell/dto";

export function useHandleExecuteChartCell() {
  const { setOutput, setExecutionState, setOutputState } =
    useChartCellExecutionStore();
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
