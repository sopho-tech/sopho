import { Button, Flex, Text } from "src/components/design-system";
import { useConversations } from "src/api/conversational_analytics";
import { useLocation, useNavigate, useParams } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { RECENT_CONVERSATIONS_LIMIT } from "src/components/ConversationalAnalytics/constants";
import { ConversationRow } from "./ConversationRow";

export const Sidebar = () => {
  const { id: activeConversationId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const { data } = useConversations({
    page: 0,
    pageSize: RECENT_CONVERSATIONS_LIMIT,
  });

  const conversations = data?.items ?? [];
  const isListViewActive =
    pathname === APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATIONS;
  const hasMore = (data?.total ?? 0) > conversations.length;

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
      {conversations.map((conversation) => (
        <ConversationRow
          key={conversation.id}
          conversation={conversation}
          isActive={
            !isListViewActive && conversation.id === activeConversationId
          }
        />
      ))}
      {hasMore && (
        <Flex marginLeft="sm" marginRight="sm" marginTop="2xs">
          <Button
            label="Show All"
            shape="rectangle"
            size="sm"
            backgroundColor="ghost"
            fullWidth
            onClick={() =>
              navigate(APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATIONS)
            }
          />
        </Flex>
      )}
    </Flex>
  );
};
