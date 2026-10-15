import { useCallback } from "react";
import ToolbarStyles from "src/css/toolbar.module.css";
import * as Toolbar from "@radix-ui/react-toolbar";
import { InlineEdit } from "src/components/design-system/InlineEdit";
import { useCell, useUpdateCell } from "src/api/cell";
import { useCanvasCapabilities } from "src/api/canvas/queries";
import {
  useExecutionPhase,
  ExecutionPhase,
  isAtLeast,
} from "src/components/Notebook/Cell/useExecutionPhase";
import { ElapsedTime } from "src/components/Notebook/ExecutionIndicator";
import { Flex } from "src/components/design-system";
import { AiEditedBadge } from "src/components/Attribution";
import styles from "src/components/Notebook/ChartCell/ChartCellToolbar/ChartCellToolbar.module.css";

export function ChartCellToolbar({ cellId }: { cellId: string }) {
  const updateCellMutation = useUpdateCell();
  const getCellQuery = useCell(cellId);
  const canEdit = useCanvasCapabilities().can_edit;
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
    [cellId, getCellQuery.data, updateCellMutation],
  );

  return (
    <Toolbar.Root className={ToolbarStyles.root} loop>
      <Flex direction="row" alignItems="center" gap="xs">
        <div className={ToolbarStyles.cellNameContainer}>
          <InlineEdit
            value={getCellQuery.data?.name ?? ""}
            onSave={handleSave}
            placeholder="Cell Name"
            defaultValue="Unnamed"
            className={styles["toolbarInlineEdit"]}
            readOnly={!canEdit}
          />
        </div>
        {getCellQuery.data && (
          <AiEditedBadge modifiedBy={getCellQuery.data.modified_by} />
        )}
      </Flex>
      {showElapsed && startedAt && <ElapsedTime startedAt={startedAt} />}
    </Toolbar.Root>
  );
}
