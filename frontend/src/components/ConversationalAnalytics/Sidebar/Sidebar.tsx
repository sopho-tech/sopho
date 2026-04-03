import { Flex, Text } from "src/components/design-system";
import { useConversations } from "src/api/conversational_analytics";
import { useParams } from "react-router";
import { ConversationRow } from "./ConversationRow";

export const Sidebar = () => {
  const { id: activeConversationId } = useParams<{ id: string }>();
  const { data: conversations } = useConversations();

  return (
    <Flex
      flex={1}
      direction="column"
      borderRight="divider"
      overflow="scrollY"
      paddingY="xs"
      as="ul"
    >
      <Flex paddingX="xs" marginLeft="sm" marginRight="sm" paddingY="2xs">
        <Text color="subtle" fontSize="sm">
          Recent Conversations
        </Text>
      </Flex>
      {conversations?.map((c) => (
        <ConversationRow
          key={c.id}
          conversation={c}
          isActive={c.id === activeConversationId}
        />
      ))}
    </Flex>
  );
};
