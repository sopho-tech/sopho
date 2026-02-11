import CellToolbarStyles from "src/components/Notebook/CellToolbar/CellToolbar.module.css";
import { Toolbar } from "src/components/design-system/Toolbar";
import { IconButton } from "src/components/design-system/IconButton/IconButton";
import { useConnections } from "src/api/connection";
import { Select } from "src/components/design-system/Select";
import { useUpdateCell, useCell } from "src/api/cell";
import { useEffect, useState } from "react";
import { ToolTip } from "src/components/design-system/ToolTip";
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

  function handleValueChange(value: string | null) {
    const cell = getCellQuery.data;
    if (!cell) throw new Error("Cell not found");

    updateCellMutation.mutate({
      cellId: cellId,
      payload: {
        ...cell,
        connection_id: value,
      },
    });
  }

  const messageElement = (
    <aside className={CellToolbarStyles.cellNameToolTipContainer}>
      <h3 className={CellToolbarStyles.cellNameToolTipHeader}>Cell Name</h3>
      <p className={CellToolbarStyles.cellNameToolTipDescription}>
        Double click to edit
      </p>
    </aside>
  );
  const toolTipTrigger = (
    <div className={CellToolbarStyles.cellNameContainer}>
      <span>{getCellQuery.data?.name}</span>
    </div>
  );

  return (
    <Toolbar loop className={CellToolbarStyles.toolbar}>
      <ToolTip messageElement={messageElement} children={toolTipTrigger} />
      <div className={CellToolbarStyles.rightSideContainer}>
        <Select
          value={initialValue?.value ?? ""}
          onValueChange={(v: string) => handleValueChange(v || null)}
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
            onClick={() => handleExecuteCell(cellId)}
          />
        </Toolbar.Button>
      </div>
    </Toolbar>
  );
}
