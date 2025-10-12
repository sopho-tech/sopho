import * as Dialog from "@radix-ui/react-dialog";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import { useConnectionsStore } from "src/components/Connection/store";
import { CloseButton } from "src/components/CloseButton/CloseButton";
import ConnectionDetailStyles from "src/components/Connection/ConnectionDetail/ConnectionDetail.module.css";
import { NewAssetButton } from "src/components/NewAssetButton/NewAssetButton";
import { StatusBadge } from "src/components/StatusBadge/StatusBadge";
import { useConnection } from "src/api/connection/queries";

export function ConnectionDetail() {
  const {
    connectionId,
    connectionDetailsPageState,
    setConnectionDetailsPageState,
  } = useConnectionsStore();

  const {
    data: connectionDetails,
    isLoading,
    isError,
    error,
  } = useConnection(connectionId);

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.LIST);
    }
  };

  return (
    <Dialog.Root
      open={
        connectionDetailsPageState === ConnectionDetailsPageStateEnum.DETAIL
      }
      onOpenChange={handleOpenChange}
    >
      <Dialog.Portal>
        <Dialog.Overlay className={ConnectionDetailStyles.dialogOverlay} />
        <Dialog.Content className={ConnectionDetailStyles.dialogContent}>
          <Dialog.Title asChild>
            <div className={ConnectionDetailStyles.dialogTitle}>
              <h4>Connection Details</h4>
              <Dialog.Close asChild>
                <CloseButton
                  onClick={() =>
                    setConnectionDetailsPageState(
                      ConnectionDetailsPageStateEnum.LIST,
                    )
                  }
                />
              </Dialog.Close>
            </div>
          </Dialog.Title>
          {isLoading && (
            <div className={ConnectionDetailStyles.loadingText}>
              Loading connection details...
            </div>
          )}
          {isError && (
            <div className={ConnectionDetailStyles.errorText}>
              Error fetching connection details:{" "}
              {error?.message || "Unknown error"}
            </div>
          )}
          {!isLoading && !isError && !connectionDetails && (
            <div className={ConnectionDetailStyles.noDetailsText}>
              No details found for this connection.
            </div>
          )}
          {connectionDetails && (
            <dl className={ConnectionDetailStyles.connectionDetailsContainer}>
              <dt>Name</dt>
              <dd>{connectionDetails.name}</dd>
              <dt>Source Type</dt>
              <dd>{connectionDetails.source_type}</dd>
              <dt>Host</dt>
              <dd>{connectionDetails.host}</dd>
              <dt>Port</dt>
              <dd>{connectionDetails.port}</dd>
              <dt>Username</dt>
              <dd>********</dd>
              <dt>Password</dt>
              <dd>********</dd>
              <dt>Database</dt>
              <dd>{connectionDetails.database}</dd>
              <dt>Description</dt>
              <dd>{connectionDetails.description}</dd>
              <dt>Created At</dt>
              <dd>{new Date(connectionDetails.created_at).toISOString()}</dd>
              <dt>Updated At</dt>
              <dd>{new Date(connectionDetails.updated_at).toISOString()}</dd>
              <dt>Status</dt>
              <dd className={ConnectionDetailStyles.statusDd}>
                <StatusBadge
                  status={connectionDetails.status}
                  text={connectionDetails.status}
                />
              </dd>
            </dl>
          )}
          <Dialog.Close asChild>
            <div className={ConnectionDetailStyles.backButton}>
              <NewAssetButton
                buttonText="Back"
                onClick={() =>
                  setConnectionDetailsPageState(
                    ConnectionDetailsPageStateEnum.LIST,
                  )
                }
              />
            </div>
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
