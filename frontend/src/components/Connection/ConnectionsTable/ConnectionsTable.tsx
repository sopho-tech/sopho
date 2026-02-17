import { useCallback, useMemo } from "react";
import ConnectionsStyles from "src/components/Connection/ConnectionsPage/ConnectionsPage.module.css";
import { StatusBadge } from "src/components/StatusBadge/StatusBadge";
import { ActionButtons } from "src/components/ActionButtons";
import { useStore } from "src/store";
import {
  ConnectionDetailsPageStateEnum,
  StatusType,
  ConnectionDto,
} from "src/components/Connection/dto";
import {
  useConnections,
  useDeleteConnection,
} from "src/api/connection/queries";
import { SophoTable, ColumnConfig } from "src/components/SophoTable/SophoTable";
import { formatTimestamp } from "src/utils/timestamp_utils";

export function ConnectionsTable() {
  const setConnectionId = useStore((state) => state.connection.setConnectionId);
  const setConnectionDetailsPageState = useStore((state) => state.connection.setConnectionDetailsPageState);
  const deleteMutation = useDeleteConnection();

  const handleViewConnection = useCallback(
    (id: string) => {
      setConnectionId(id);
      setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.DETAIL);
    },
    [setConnectionId, setConnectionDetailsPageState]
  );

  const handleEditConnection = useCallback(
    (id: string) => {
      setConnectionId(id);
      setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.EDIT);
    },
    [setConnectionId, setConnectionDetailsPageState]
  );

  const handleDelete = useCallback(
    (id: string) => {
      if (confirm("Are you sure you want to delete this connection?")) {
        deleteMutation.mutate(id);
      }
    },
    [deleteMutation]
  );

  const columns: ColumnConfig<ConnectionDto>[] = useMemo(() => [
    {
      key: "name",
      header: "Name",
      type: "accessor",
      accessor: "name",
      size: 150,
    },
    {
      key: "source_type",
      header: "Source Type",
      type: "accessor",
      accessor: "source_type",
      size: 150,
    },
    {
      key: "status",
      header: "Status",
      type: "accessor",
      accessor: "status",
      cell: (props) => {
        const statusValue = props.getValue();
        let statusEnumEntry: StatusType;

        if (statusValue === StatusType.Active) {
          statusEnumEntry = StatusType.Active;
        } else if (statusValue === StatusType.Inactive) {
          statusEnumEntry = StatusType.Inactive;
        } else if (statusValue === StatusType.Failed) {
          statusEnumEntry = StatusType.Failed;
        } else {
          throw new Error(`Invalid status value: ${statusValue}`);
        }

        return <StatusBadge status={statusEnumEntry} text={statusValue} />;
      },
      size: 150,
    },
    {
      key: "description",
      header: "Description",
      type: "accessor",
      accessor: "description",
      size: 300,
    },
    {
      key: "created_at",
      header: "Created On",
      type: "accessor",
      accessor: "created_at",
      cell: (props) => formatTimestamp(props.getValue() as string | null),
      size: 210,
    },
    {
      key: "actions",
      header: "Actions",
      type: "display",
      cell: (props) => (
        <ActionButtons
          connectionId={props.row.original.id}
          onViewClick={handleViewConnection}
          onEditClick={handleEditConnection}
          onDeleteClick={handleDelete}
        />
      ),
      size: 100,
    },
  ], [handleViewConnection, handleEditConnection, handleDelete]);

  const { data: connectionsData, isLoading, isError } = useConnections();

  const tableData = useMemo(
    () => connectionsData ?? [],
    [connectionsData]
  );

  return (
    <div className={ConnectionsStyles.connectionsTable}>
      <SophoTable
        columns={columns}
        data={tableData}
        isLoading={isLoading}
        isError={isError}
      />
    </div>
  );
}
