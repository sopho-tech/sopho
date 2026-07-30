import { useCallback } from "react";
import { useNavigate } from "react-router";
import { Button, Flex, Text } from "src/components/design-system";
import { APP_ROUTES } from "src/constants/app_routes";

export const ConversationLimitNotice = () => {
  const navigate = useNavigate();

  const handleStartNewConversation = useCallback(() => {
    navigate(APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.INDEX);
  }, [navigate]);

  return (
    <Flex
      direction="column"
      alignItems="center"
      gap="sm"
      paddingY="md"
      paddingX="md"
      borderRadius="lg"
      backgroundColor="grey"
    >
      <Text fontSize="sm" color="darkGrey">
        This conversation has reached its message limit.
      </Text>
      <Button
        label="Start a new conversation"
        shape="pill"
        backgroundColor="accent"
        size="sm"
        leadingIconName="add"
        onClick={handleStartNewConversation}
      />
    </Flex>
  );
};
