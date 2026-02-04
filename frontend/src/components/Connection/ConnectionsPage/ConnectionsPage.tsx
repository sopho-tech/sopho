import { useConnectionsStore } from "src/components/Connection/store";
import { ConnectionsTable } from "src/components/Connection/ConnectionsTable/ConnectionsTable";
import { ConnectionDetail } from "src/components/Connection/ConnectionDetail/ConnectionDetail";
import { ConnectionEdit } from "src/components/Connection/ConnectionEdit/ConnectionEdit";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import { ConnectionNew } from "src/components/Connection/ConnectionNew";
import { Button } from "src/components/design-system/Button/Button";
import { Flex } from "src/components/design-system/Flex/Flex";

export function Connections() {
  const { connectionDetailsPageState, setConnectionDetailsPageState } =
    useConnectionsStore();

  return (
    <Flex direction="column" flex="grow" gap="md" overflow="hidden">
      <Flex
        direction="row-reverse"
        justifyContent="space-between"
        alignItems="center"
      >
        <Button
          label="New"
          leadingIconName="add"
          backgroundColor="accent"
          shape="rectangle"
          size="md"
          onClick={() =>
            setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.NEW)
          }
        />
      </Flex>
      {connectionDetailsPageState === ConnectionDetailsPageStateEnum.NEW && (
        <ConnectionNew />
      )}
      {connectionDetailsPageState === ConnectionDetailsPageStateEnum.DETAIL && (
        <ConnectionDetail />
      )}
      {connectionDetailsPageState === ConnectionDetailsPageStateEnum.EDIT && (
        <ConnectionEdit />
      )}
      <ConnectionsTable />
    </Flex>
  );
}
