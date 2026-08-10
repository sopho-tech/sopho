import { useCallback } from "react";
import styles from "src/components/Notebook/Cell/Cell.module.css";
import activeCellStyles from "src/components/Notebook/activeCell.module.css";
import { CellEditor } from "src/components/Notebook/CellEditor/CellEditor";
import { CellToolbar } from "src/components/Notebook/CellToolbar/CellToolbar";
import { CellOutput } from "src/components/Notebook/CellOutput/CellOutput";
import { useCellContainerClassName } from "src/components/Notebook/hooks";
import {
  useExecutionPhase,
  ExecutionPhase,
  isAtLeast,
} from "src/components/Notebook/Cell/useExecutionPhase";
import { QueryProgressBar } from "src/components/Notebook/ExecutionIndicator";
import { Box } from "src/components/design-system";
import { useStore } from "src/store";

export function Cell({ cell_id }: { cell_id: string }) {
  const styleClassName = useCellContainerClassName(
    cell_id,
    styles.cellContainer,
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
      className={styleClassName}
      onFocus={handleActivate}
      onClick={handleActivate}
      revealChildrenOnHover
      aria-busy={isRunning}
    >
      <Box>
        <CellToolbar cellId={cell_id} />
        <CellEditor cellId={cell_id} />
      </Box>
      {showProgressBar && <QueryProgressBar />}
      <CellOutput cellId={cell_id} />
    </Box>
  );
}
