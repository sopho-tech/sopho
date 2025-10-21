import ConnectionsStyles from "src/components/Connection/ConnectionsPage/ConnectionsPage.module.css";
import { StatusBadge } from "src/components/StatusBadge/StatusBadge";
import { ActionButtons } from "src/components/ActionButtons";
import { useConnectionsStore } from "src/components/Connection/store";
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
  const { setConnectionDetailsPageState, setConnectionId } =
    useConnectionsStore();
  const deleteMutation = useDeleteConnection();

  const handleViewConnection = async (id: string) => {
    setConnectionId(id);
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.DETAIL);
  };

  const handleEditConnection = (id: string) => {
    setConnectionId(id);
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.EDIT);
  };

  const handleDelete = (id: string) => {
    if (confirm("Are you sure you want to delete this connection?")) {
      deleteMutation.mutate(id);
    }
  };

  const columns: ColumnConfig<ConnectionDto>[] = [
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
      cell: (props) => formatTimestamp(props.getValue()),
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
  ];

  const { data: connectionsData, isLoading, isError } = useConnections();

  return (
    <div className={ConnectionsStyles.connectionsTable}>
      <SophoTable
        columns={columns}
        data={connectionsData ?? []}
        isLoading={isLoading}
        isError={isError}
      />
    </div>
  );
}
