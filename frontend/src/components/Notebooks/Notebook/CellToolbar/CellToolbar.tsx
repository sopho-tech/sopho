import CellToolbarStyles from "src/components/Notebooks/Notebook/CellToolbar/CellToolbar.module.css";
import * as Toolbar from "@radix-ui/react-toolbar";
import { ExecuteButton } from "src/components/ExecuteButton/ExecuteButton";
import { useConnections } from "src/api/connection";
import { SophoSelect } from "src/components/SophoSelect";
import { useUpdateCell, useCell, useExecuteCell } from "src/api/cell";
import { useEffect, useState } from "react";
import {
  CellOutputState,
  useCellOutputStore,
} from "src/components/Notebooks/Notebook/Cell";
import { ExecutionState } from "src/components/Notebooks/Notebook/Cell";
import { SophoToolTip } from "src/components/SophoToolTip";

export function CellToolbar({ cellId }: { cellId: string }) {
  const query = useConnections();
  const updateCellMutation = useUpdateCell();
  const executeCellMutation = useExecuteCell();
  const getCellQuery = useCell(cellId);
  const [options, setOptions] = useState<
    { label: string; value: string }[] | undefined
  >(undefined);
  const [initialValue, setInitialValue] = useState<
    { label: string; value: string } | undefined
  >(undefined);
  const { setOutput, setExecutionState, setOutputState } = useCellOutputStore();

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
    <Toolbar.Root className={CellToolbarStyles.root} loop>
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
          <ExecuteButton onClick={handleExecute} />
        </Toolbar.Button>
      </div>
    </Toolbar.Root>
  );
}
