import { useCallback, useMemo, useState } from "react";
import { CreatedByFilter } from "src/api/dto";
import { CreatedByFilterControl } from "src/components/Attribution";
import { describeActor } from "src/components/Attribution/describeActor";
import { CREATED_BY_LABEL } from "src/components/Attribution/constants";
import { Flex } from "src/components/design-system";
import ConnectionsStyles from "src/components/Connection/ConnectionsPage/ConnectionsPage.module.css";
import { StatusBadge } from "src/components/StatusBadge/StatusBadge";
import { ActionButtons } from "src/components/ActionButtons";
import { useStore } from "src/store";
import {
  ConnectionDetailsPageStateEnum,
  StatusType,
  ConnectionDto,
} from "src/components/Connection/dto";
import { UNAVAILABLE_CONNECTION_STATUS_LABEL } from "src/components/Connection/constants";
import {
  useConnections,
  useDeleteConnection,
} from "src/api/connection/queries";
import { Table, ColumnConfig } from "src/components/Table/Table";
import { formatTimestamp } from "src/utils/timestamp_utils";
import { ConnectionsEmptyState } from "src/components/Home/components/EmptyState";
import { useNavigate, generatePath } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { useTenantPath } from "src/utils/tenant";

export function ConnectionsTable() {
  const navigate = useNavigate();
  const toTenantPath = useTenantPath();
  const setConnectionId = useStore((state) => state.connection.setConnectionId);
  const setConnectionDetailsPageState = useStore(
    (state) => state.connection.setConnectionDetailsPageState,
  );
  const deleteMutation = useDeleteConnection();
  const [createdBy, setCreatedBy] = useState<CreatedByFilter | null>(null);
  const {
    data: connectionsData,
    isLoading,
    isError,
  } = useConnections(createdBy);
  const tableData = useMemo(() => connectionsData ?? [], [connectionsData]);

  const handleViewConnection = useCallback(
    (id: string) => {
      setConnectionId(id);
      setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.DETAIL);
    },
    [setConnectionId, setConnectionDetailsPageState],
  );

  const handleEditConnection = useCallback(
    (id: string) => {
      void navigate(
        toTenantPath(generatePath(APP_ROUTES.CONNECTION_EDIT, { id })),
      );
    },
    [navigate, toTenantPath],
  );

  const handleDelete = useCallback(
    (id: string) => {
      if (confirm("Are you sure you want to delete this connection?")) {
        deleteMutation.mutate(id);
      }
    },
    [deleteMutation],
  );

  const handleCreateConnection = useCallback(() => {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.NEW);
  }, [setConnectionDetailsPageState]);

  const columns: ColumnConfig<ConnectionDto>[] = useMemo(
    () => [
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
        accessor: "source_type_display_name",
        size: 150,
      },
      {
        key: "status",
        header: "Status",
        type: "accessor",
        accessor: "status",
        cell: (props) => {
          if (!props.row.original.is_source_type_available) {
            return (
              <StatusBadge
                status={StatusType.Inactive}
                text={UNAVAILABLE_CONNECTION_STATUS_LABEL}
              />
            );
          }
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
        key: "created_by",
        header: CREATED_BY_LABEL,
        type: "display",
        cell: (props) => describeActor(props.row.original.created_by),
        size: 180,
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
            capabilities={props.row.original.capabilities}
            onViewClick={handleViewConnection}
            onEditClick={handleEditConnection}
            onDeleteClick={handleDelete}
          />
        ),
        size: 100,
      },
    ],
    [handleViewConnection, handleEditConnection, handleDelete],
  );

  const render = () => {
    if (tableData.length > 0 || createdBy !== null) {
      return (
        <Flex direction="column" gap="md">
          <Flex direction="row">
            <CreatedByFilterControl
              value={createdBy}
              onValueChange={setCreatedBy}
            />
          </Flex>
          <Table
            columns={columns}
            data={tableData}
            isLoading={isLoading}
            isError={isError}
          />
        </Flex>
      );
    }
    return (
      <ConnectionsEmptyState onCreateConnection={handleCreateConnection} />
    );
  };

  return <div className={ConnectionsStyles.connectionsTable}>{render()}</div>;
}
