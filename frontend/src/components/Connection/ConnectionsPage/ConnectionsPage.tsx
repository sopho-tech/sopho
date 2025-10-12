import ConnectionsStyles from "src/components/Connection/ConnectionsPage/ConnectionsPage.module.css";
import { NewAssetButton } from "src/components/NewAssetButton";
import { CreateNewConnection } from "src/components/Connection/CreateNewConnection";
import { useConnectionsStore } from "src/components/Connection/store";
import { ConnectionsTable } from "src/components/Connection/ConnectionsTable/ConnectionsTable";
import { ConnectionDetail } from "src/components/Connection/ConnectionDetail/ConnectionDetail";
import { ConnectionEdit } from "src/components/Connection/ConnectionEdit/ConnectionEdit";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";

export function Connections() {
  const { connectionDetailsPageState, setConnectionDetailsPageState } =
    useConnectionsStore();

  return (
    <div className={ConnectionsStyles.connectionsContainer}>
      <div className={ConnectionsStyles.connectionsHeader}>
        <NewAssetButton
          buttonText="New Connection"
          onClick={() =>
            setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.NEW)
          }
        />
      </div>
      {connectionDetailsPageState === ConnectionDetailsPageStateEnum.NEW && (
        <CreateNewConnection />
      )}
      {connectionDetailsPageState === ConnectionDetailsPageStateEnum.DETAIL && (
        <ConnectionDetail />
      )}
      {connectionDetailsPageState === ConnectionDetailsPageStateEnum.EDIT && (
        <ConnectionEdit />
      )}
      <ConnectionsTable />
    </div>
  );
}
