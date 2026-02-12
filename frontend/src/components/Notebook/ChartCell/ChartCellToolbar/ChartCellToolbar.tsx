import ToolbarStyles from "src/css/toolbar.module.css";
import * as Toolbar from "@radix-ui/react-toolbar";
import { IconButton } from "src/components/design-system/IconButton/IconButton";
import { InlineEdit } from "src/components/design-system/InlineEdit";
import { useCell, useExecuteCell, useUpdateCell } from "src/api/cell";
import { CellOutputState, ExecutionState } from "src/components/Notebook/Cell";
import { useStore } from "src/store";
import styles from "./ChartCellToolbar.module.css";

export function ChartCellToolbar({ cellId }: { cellId: string }) {
  const executeCellMutation = useExecuteCell();
  const updateCellMutation = useUpdateCell();
  const getCellQuery = useCell(cellId);
  const setOutput = useStore((state) => state.cell.setOutput);
  const setExecutionState = useStore((state) => state.cell.setExecutionState);
  const setOutputState = useStore((state) => state.cell.setOutputState);

  function handleExecute() {
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

  return (
    <Toolbar.Root className={ToolbarStyles.root} loop>
      <div className={ToolbarStyles.cellNameContainer}>
        <InlineEdit
          value={getCellQuery.data?.name ?? ""}
          onSave={(name) => {
            const cell = getCellQuery.data;
            if (cell) {
              updateCellMutation.mutate({
                cellId,
                payload: { ...cell, name },
              });
            }
          }}
          placeholder="Cell Name"
          defaultValue="Unnamed"
          className={styles["toolbar__inlineEdit"]}
        />
      </div>
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
