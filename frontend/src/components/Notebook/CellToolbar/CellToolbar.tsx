import CellToolbarStyles from "src/components/Notebook/CellToolbar/CellToolbar.module.css";
import { Toolbar } from "src/components/design-system/Toolbar";
import { IconButton } from "src/components/design-system/IconButton/IconButton";
import { InlineEdit } from "src/components/design-system/InlineEdit";
import { useConnections } from "src/api/connection";
import { Select } from "src/components/design-system/Select";
import { useUpdateCell, useCell, useClearCellOutput } from "src/api/cell";
import { useCallback, useEffect, useState } from "react";
import { useHandleRunCell } from "src/components/Notebook/Cell";
import { getSelectedText } from "src/components/Notebook/CellEditor/editorRegistry";
import {
  useExecutionPhase,
  ExecutionPhase,
  isAtLeast,
} from "src/components/Notebook/Cell/useExecutionPhase";
import { ElapsedTime } from "src/components/Notebook/ExecutionIndicator";
import { Flex, Kbd, Spinner } from "src/components/design-system";
import {
  KEYBOARD_SHORTCUTS,
  getShortcutDisplayString,
} from "src/utils/keyboard_shortcuts";

function ExecuteTooltipContent({ cellId }: { cellId: string }) {
  return (
    <Flex direction="row" alignItems="center" gap="md">
      <span>{getSelectedText(cellId) ? "Run selection" : "Execute cell"}</span>
      <Kbd>
        {getShortcutDisplayString(KEYBOARD_SHORTCUTS.EXECUTE_NOTEBOOK_CELL)}
      </Kbd>
    </Flex>
  );
}

export function CellToolbar({ cellId }: { cellId: string }) {
  const clearCellOutput = useClearCellOutput();
  const query = useConnections();
  const updateCellMutation = useUpdateCell();
  const getCellQuery = useCell(cellId);
  const [options, setOptions] = useState<
    { label: string; value: string }[] | undefined
  >(undefined);
  const [initialValue, setInitialValue] = useState<
    { label: string; value: string } | undefined
  >(undefined);
  const handleRunCell = useHandleRunCell();
  const { isRunning, phase, startedAt } = useExecutionPhase(cellId);
  const showElapsed = isAtLeast(phase, ExecutionPhase.EXTENDED);

  useEffect(() => {
    if (!query.data || !getCellQuery.data) return;

    const connectionOptions = query.data.map((connection) => ({
      label: connection.name,
      value: connection.id,
    }));

    const cellConnectionId = getCellQuery.data.connection_id;
    const foundConnection = query.data.find(
      (connection) => connection.id === cellConnectionId
    );

    const newInitialValue =
      foundConnection && cellConnectionId
        ? { label: foundConnection.name, value: cellConnectionId }
        : undefined;

    setOptions(connectionOptions);
    setInitialValue(newInitialValue);
  }, [query.data, getCellQuery.data]);

  const handleValueChange = useCallback(
    (value: string | null) => {
      const cell = getCellQuery.data;
      if (!cell) throw new Error("Cell not found");

      updateCellMutation.mutate({
        cellId: cellId,
        payload: {
          ...cell,
          connection_id: value,
        },
      });
    },
    [cellId, getCellQuery.data, updateCellMutation]
  );

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

  const handleExecute = useCallback(() => {
    handleRunCell(cellId);
  }, [cellId, handleRunCell]);

  const handleClearOutput = useCallback(() => {
    clearCellOutput(cellId);
  }, [cellId, clearCellOutput]);

  const handleSelectChange = useCallback(
    (v: string) => handleValueChange(v || null),
    [handleValueChange]
  );

  return (
    <Toolbar loop className={CellToolbarStyles.toolbar}>
      <div className={CellToolbarStyles.cellNameContainer}>
        <InlineEdit
          value={getCellQuery.data?.name ?? ""}
          onSave={handleSave}
          placeholder="Cell Name"
          defaultValue="Unnamed"
          className={CellToolbarStyles["toolbar__inlineEdit"]}
        />
      </div>
      <Flex
        direction="row"
        alignItems="center"
        gap="md"
        revealOnHover={!isRunning}
      >
        {showElapsed && startedAt && <ElapsedTime startedAt={startedAt} />}
        <Select
          value={initialValue?.value ?? ""}
          onValueChange={handleSelectChange}
        >
          <Toolbar.Button asChild>
            <Select.Trigger
              placeholder="Select a connection"
              className={CellToolbarStyles["toolbar__select-trigger"]}
            />
          </Toolbar.Button>
          <Select.Content>
            <Select.Group>
              <Select.Label>Connections</Select.Label>
              {(options ?? []).map((opt) => (
                <Select.Item key={opt.value} value={opt.value}>
                  {opt.label}
                </Select.Item>
              ))}
            </Select.Group>
          </Select.Content>
        </Select>
        <Toolbar.Button asChild>
          <Flex gap="xs" alignItems="center">
            <IconButton
              type="clear"
              backgroundColor="transparent"
              iconColor="red"
              onClick={handleClearOutput}
              disabled={isRunning}
              tooltip={{
                content: (
                  <Flex direction="row" alignItems="center" gap="md">
                    <span>Clear cell output</span>
                    <Kbd>
                      {getShortcutDisplayString(
                        KEYBOARD_SHORTCUTS.CLEAR_NOTEBOOK_CELL
                      )}
                    </Kbd>
                  </Flex>
                ),
              }}
            />
            {isRunning ? (
              <Spinner size="md" color="green" />
            ) : (
              <IconButton
                type="play"
                backgroundColor="transparent"
                iconColor="green"
                onClick={handleExecute}
                tooltip={{
                  content: <ExecuteTooltipContent cellId={cellId} />,
                }}
              />
            )}
          </Flex>
        </Toolbar.Button>
      </Flex>
    </Toolbar>
  );
}
