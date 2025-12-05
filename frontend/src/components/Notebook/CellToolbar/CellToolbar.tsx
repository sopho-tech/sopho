import ToolbarStyles from "src/css/toolbar.module.css";
import CellToolbarStyles from "src/components/Notebook/CellToolbar/CellToolbar.module.css";
import * as Toolbar from "@radix-ui/react-toolbar";
import { IconButton } from "src/components/design-system/IconButton/IconButton";
import { useConnections } from "src/api/connection";
import { SophoSelect } from "src/components/SophoSelect";
import { useUpdateCell, useCell } from "src/api/cell";
import { useEffect, useState } from "react";
import { SophoToolTip } from "src/components/SophoToolTip";
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
      <SophoToolTip messageElement={messageElement} children={toolTipTrigger} />
      <div className={CellToolbarStyles.rightSideContainer}>
        <Toolbar.Button asChild>
          <SophoSelect
            groupName="Connections"
            initialValue={initialValue}
            onValueChange={handleValueChange}
            options={options}
          />
        </Toolbar.Button>
        <Toolbar.Button asChild>
          <IconButton
            type="play"
            backgroundColor="transparent"
            iconColor="green"
            onClick={() => handleExecuteCell(cellId)}
          />
        </Toolbar.Button>
      </div>
    </Toolbar.Root>
  );
}
