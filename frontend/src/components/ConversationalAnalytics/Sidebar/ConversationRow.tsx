import { hoverHighlightTransition } from "src/components/design-system/animation";
import { Flex, Icon, MotionFlex, Text } from "src/components/design-system";
import { Spinner } from "src/components/design-system/Spinner";
import { ConversationDto } from "src/api/conversational_analytics";
import { useConversationStream } from "src/components/ConversationalAnalytics/ConversationStreamContext";
import { ConversationActionsMenu } from "src/components/ConversationalAnalytics/ConversationActionsMenu";
import { useCallback, useState } from "react";
import { generatePath, useNavigate } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";

type ConversationRowProps = {
  conversation: ConversationDto;
  isActive: boolean;
};

export const ConversationRow = ({
  conversation,
  isActive,
}: ConversationRowProps) => {
  const navigate = useNavigate();
  const { isStreaming } = useConversationStream(conversation.id);
  const [hovered, setHovered] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);

  const triggerVisible = hovered || menuOpen;

  const handleClick = useCallback(() => {
    navigate(
      generatePath(APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION, {
        id: conversation.id,
      }),
    );
  }, [navigate, conversation.id]);

  return (
    <MotionFlex
      key={conversation.id}
      justifyContent="space-between"
      alignItems="center"
      paddingX="xs"
      paddingY="2xs"
      marginLeft="sm"
      marginRight="sm"
      as="li"
      backgroundColor="default"
      borderRadius="lg"
      whileHover={{ backgroundColor: "var(--color-grey-200)" }}
      transition={hoverHighlightTransition}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
      onClick={handleClick}
      sx={{
        cursor: "pointer",
        ...(isActive || menuOpen
          ? { backgroundColor: "var(--color-grey-200)" }
          : {}),
      }}
    >
      <Flex alignItems="center" gap="xs" overflow="hidden">
        <Text truncate color="darkGrey" fontSize="sm">
          {conversation.name}
        </Text>
        {isStreaming && !triggerVisible && <Spinner size="sm" color="grey" />}
      </Flex>
      <ConversationActionsMenu
        conversation={conversation}
        onOpenChange={setMenuOpen}
      >
        <button
          type="button"
          aria-label="Conversation actions"
          onClick={(e) => e.stopPropagation()}
          style={{
            opacity: triggerVisible ? 1 : 0,
            pointerEvents: triggerVisible ? "auto" : "none",
            ...(triggerVisible
              ? { flexShrink: 0 }
              : {
                  width: 0,
                  minWidth: 0,
                  padding: 0,
                  margin: 0,
                  border: "none",
                  overflow: "hidden",
                }),
          }}
        >
          <Icon type="more_vert" color="grey" size="md" />
        </button>
      </ConversationActionsMenu>
    </MotionFlex>
  );
};
