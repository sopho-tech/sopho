import { Flex } from "src/components/design-system";
import { Outlet } from "react-router";

export const ConversationSection = () => {
  return (
    <Flex flex={5} direction="column" height="100%" paddingX="lg">
      <Outlet />
    </Flex>
  );
};
