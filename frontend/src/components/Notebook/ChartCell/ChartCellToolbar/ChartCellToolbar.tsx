import { useCallback } from "react";
import ToolbarStyles from "src/css/toolbar.module.css";
import * as Toolbar from "@radix-ui/react-toolbar";
import { InlineEdit } from "src/components/design-system/InlineEdit";
import { useCell, useUpdateCell } from "src/api/cell";
import {
  useExecutionPhase,
  ExecutionPhase,
  isAtLeast,
} from "src/components/Notebook/Cell/useExecutionPhase";
import { ElapsedTime } from "src/components/Notebook/ExecutionIndicator";
import styles from "./ChartCellToolbar.module.css";

export function ChartCellToolbar({ cellId }: { cellId: string }) {
  const updateCellMutation = useUpdateCell();
  const getCellQuery = useCell(cellId);
  const { phase, startedAt } = useExecutionPhase(cellId);
  const showElapsed = isAtLeast(phase, ExecutionPhase.EXTENDED);

  const handleSave = useCallback(
    (name: string) => {
      const cell = getCellQuery.data;
      if (cell) {
        updateCellMutation.mutate({
          cellId,
          payload: { ...cell, name },
        });
      }
    },
    [cellId, getCellQuery.data, updateCellMutation]
  );

  return (
    <Toolbar.Root className={ToolbarStyles.root} loop>
      <div className={ToolbarStyles.cellNameContainer}>
        <InlineEdit
          value={getCellQuery.data?.name ?? ""}
          onSave={handleSave}
          placeholder="Cell Name"
          defaultValue="Unnamed"
          className={styles["toolbar__inlineEdit"]}
        />
      </div>
      {showElapsed && startedAt && <ElapsedTime startedAt={startedAt} />}
    </Toolbar.Root>
  );
}
