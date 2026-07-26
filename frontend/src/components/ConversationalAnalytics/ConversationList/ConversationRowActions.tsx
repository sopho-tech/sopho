import { useState } from "react";
import { Flex, Icon, Text } from "src/components/design-system";
import { rowHoverClasses } from "src/components/SophoTable";
import { ConversationListItemDto } from "src/api/conversational_analytics";
import { ConversationActionsMenu } from "src/components/ConversationalAnalytics/ConversationActionsMenu";
import { formatDate } from "src/utils/timestamp_utils";

type ConversationRowActionsProps = {
  conversation: ConversationListItemDto;
  isSelectMode: boolean;
  onEnterSelectMode: (conversationId: string) => void;
};

export const ConversationRowActions = ({
  conversation,
  isSelectMode,
  onEnterSelectMode,
}: ConversationRowActionsProps) => {
  const [menuOpen, setMenuOpen] = useState(false);

  const updatedAt = (
    <Flex sx={{ whiteSpace: "nowrap" }}>
      <Text color="subtle" fontSize="xs">
        {formatDate(conversation.updated_at)}
      </Text>
    </Flex>
  );

  if (isSelectMode) {
    return (
      <Flex justifyContent="flex-end" alignItems="center">
        {updatedAt}
      </Flex>
    );
  }

  return (
    <Flex position="relative" alignItems="center" justifyContent="flex-end">
      <Flex
        className={rowHoverClasses.hide}
        alignItems="center"
        sx={menuOpen ? { opacity: 0 } : undefined}
      >
        {updatedAt}
      </Flex>

      <Flex
        className={rowHoverClasses.reveal}
        position="absolute"
        right={0}
        alignItems="center"
        sx={menuOpen ? { opacity: 1, pointerEvents: "auto" } : undefined}
      >
        <ConversationActionsMenu
          conversation={conversation}
          onOpenChange={setMenuOpen}
          onSelect={() => onEnterSelectMode(conversation.id)}
        >
          <button
            type="button"
            aria-label="Conversation actions"
            onClick={(event) => event.stopPropagation()}
            style={{
              display: "flex",
              background: "none",
              border: "none",
              cursor: "pointer",
            }}
          >
            <Icon type="more_vert" color="grey" size="md" />
          </button>
        </ConversationActionsMenu>
      </Flex>
    </Flex>
  );
};
