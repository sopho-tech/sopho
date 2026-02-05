import ToolbarStyles from "src/css/toolbar.module.css";
import * as Toolbar from "@radix-ui/react-toolbar";
import { IconButton } from "src/components/design-system/IconButton/IconButton";
import { useCell, useExecuteCell } from "src/api/cell";
import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebook/Cell";
import { ExecutionState } from "src/components/Notebook/Cell";
import { ToolTip } from "src/components/design-system/ToolTip";

export function ChartCellToolbar({ cellId }: { cellId: string }) {
  const executeCellMutation = useExecuteCell();
  const getCellQuery = useCell(cellId);

  function handleExecute() {
    const { setOutput, setExecutionState, setOutputState } =
      useCellOutputStore.getState();
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
  }

  const messageElement = (
    <aside className={ToolbarStyles.cellNameToolTipContainer}>
      <h3 className={ToolbarStyles.cellNameToolTipHeader}>Cell Name</h3>
      <p className={ToolbarStyles.cellNameToolTipDescription}>
        Double click to edit
      </p>
    </aside>
  );

  const toolTipTrigger = (
    <div className={ToolbarStyles.cellNameContainer}>
      <span>{getCellQuery.data?.name}</span>
    </div>
  );

  return (
    <Toolbar.Root className={ToolbarStyles.root} loop>
      <ToolTip messageElement={messageElement} children={toolTipTrigger} />
      <Toolbar.Button asChild>
        <IconButton
          type="play"
          backgroundColor="transparent"
          iconColor="green"
          onClick={() => handleExecute()}
        />
      </Toolbar.Button>
    </Toolbar.Root>
  );
}
