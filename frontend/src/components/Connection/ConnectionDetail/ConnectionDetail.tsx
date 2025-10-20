import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import { useConnectionsStore } from "src/components/Connection/store";
import { StatusBadge } from "src/components/StatusBadge/StatusBadge";
import { useConnection } from "src/api/connection/queries";
import { SophoFormDetails } from "src/components/SophoFormDetails/SophoFormDetails";
import { SophoDialog } from "src/components/SophoDialog/SophoDialog";
import { Fragment } from "react/jsx-runtime";
import { formatTimestamp } from "src/utils/timestamp_utils";

export function ConnectionDetail() {
  const {
    connectionId,
    connectionDetailsPageState,
    setConnectionDetailsPageState,
  } = useConnectionsStore();

  const { data: connectionDetails } = useConnection(connectionId);

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    }
  };

  const handleDialogClose = () => {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
  };

  const dialogContent = (
    <Fragment>
      {connectionDetails && (
        <SophoFormDetails
          items={[
            { label: "Name", value: connectionDetails.name },
            { label: "Source Type", value: connectionDetails.source_type },
            { label: "Host", value: connectionDetails.host },
            { label: "Port", value: connectionDetails.port },
            { label: "Username", value: "********" },
            { label: "Password", value: "********" },
            { label: "Database", value: connectionDetails.database },
            { label: "Description", value: connectionDetails.description },
            {
              label: "Created On",
              value: formatTimestamp(connectionDetails.created_at),
            },
            {
              label: "Last Modified",
              value: formatTimestamp(connectionDetails.updated_at),
            },
            {
              label: "Status",
              value: (
                <StatusBadge
                  status={connectionDetails.status}
                  text={connectionDetails.status}
                />
              ),
            },
          ]}
        />
      )}
    </Fragment>
  );

  return (
    <SophoDialog
      shouldOpenDialog={
        connectionDetailsPageState === ConnectionDetailsPageStateEnum.DETAIL
      }
      handleOnOpenChange={handleOpenChange}
      handleDialogClose={handleDialogClose}
      title="Connection Details"
      info={dialogContent}
    />
  );
}
