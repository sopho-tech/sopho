import { useCallback } from "react";
import { useStore } from "src/store";
import { ConnectionsTable } from "src/components/Connection/ConnectionsTable/ConnectionsTable";
import { ConnectionDetail } from "src/components/Connection/ConnectionDetail/ConnectionDetail";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import { Button } from "src/components/design-system/Button/Button";
import { Flex } from "src/components/design-system/Flex/Flex";
import { useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";

export function Connections() {
  const navigate = useNavigate();
  const connectionDetailsPageState = useStore(
    (state) => state.connection.connectionDetailsPageState
  );
  const setConnectionDetailsPageState = useStore(
    (state) => state.connection.setConnectionDetailsPageState
  );

  const handleNewClick = useCallback(() => {
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.NEW);
    navigate(APP_ROUTES.CONNECTION_NEW);
  }, [setConnectionDetailsPageState, navigate]);

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
          onClick={handleNewClick}
        />
      </Flex>
      {connectionDetailsPageState === ConnectionDetailsPageStateEnum.DETAIL && (
        <ConnectionDetail />
      )}
      <ConnectionsTable />
    </Flex>
  );
}
