import CellToolbarStyles from "src/components/Notebook/CellToolbar/CellToolbar.module.css";
import { Toolbar } from "src/components/design-system/Toolbar";
import { IconButton } from "src/components/design-system/IconButton/IconButton";
import { InlineEdit } from "src/components/design-system/InlineEdit";
import { useConnections } from "src/api/connection";
import { Select } from "src/components/design-system/Select";
import { useUpdateCell, useCell } from "src/api/cell";
import { useCallback, useEffect, useState } from "react";
import { useHandleExecuteCell } from "src/components/Notebook/Cell";

export function CellToolbar({ cellId }: { cellId: string }) {
  const query = useConnections();
  const updateCellMutation = useUpdateCell();
  const getCellQuery = useCell(cellId);
  const [options, setOptions] = useState<
    { label: string; value: string }[] | undefined
  >(undefined);
  const [initialValue, setInitialValue] = useState<
    { label: string; value: string } | undefined
  >(undefined);
  const handleExecuteCell = useHandleExecuteCell();

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

  const handleExecuteClick = useCallback(() => {
    handleExecuteCell(cellId);
  }, [cellId, handleExecuteCell]);

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
      <div className={CellToolbarStyles.rightSideContainer}>
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
          <IconButton
            type="play"
            backgroundColor="transparent"
            iconColor="green"
            onClick={handleExecuteClick}
          />
        </Toolbar.Button>
      </div>
    </Toolbar>
  );
}
