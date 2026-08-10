import { useCallback } from "react";
import ChartCellStyles from "src/components/Notebook/ChartCell/ChartCell.module.css";
import activeCellStyles from "src/components/Notebook/activeCell.module.css";
import { ChartCellToolbar } from "src/components/Notebook/ChartCell/ChartCellToolbar/ChartCellToolbar";
import { CellOutput } from "src/components/Notebook/ChartCell/CellOutput/CellOutput";
import { CellEditor } from "src/components/Notebook/ChartCell/CellEditor";
import { useCellContainerClassName } from "src/components/Notebook/hooks";
import {
  useExecutionPhase,
  ExecutionPhase,
  isAtLeast,
} from "src/components/Notebook/Cell/useExecutionPhase";
import { QueryProgressBar } from "src/components/Notebook/ExecutionIndicator";
import { Box } from "src/components/design-system";
import { useStore } from "src/store";

export function ChartCell({ cell_id }: { cell_id: string }) {
  const containerClassName = useCellContainerClassName(
    cell_id,
    ChartCellStyles.container,
    activeCellStyles.activeCell
  );
  const setActiveCellId = useStore((state) => state.notebook.setActiveCellId);

  const handleActivate = useCallback(() => {
    setActiveCellId(cell_id);
  }, [cell_id, setActiveCellId]);

  const { isRunning, phase } = useExecutionPhase(cell_id);

  const showProgressBar = isAtLeast(phase, ExecutionPhase.VISIBLE);

  return (
    <Box
      id={cell_id}
      className={containerClassName}
      onFocus={handleActivate}
      onClick={handleActivate}
      aria-busy={isRunning}
    >
      <ChartCellToolbar cellId={cell_id} />
      {showProgressBar && <QueryProgressBar />}
      <Box className={ChartCellStyles.editorAndOutputContainer}>
        <CellEditor cellId={cell_id} />
        <CellOutput cellId={cell_id} />
      </Box>
    </Box>
  );
}
