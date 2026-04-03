import { useParams } from "react-router";
import {
  ConversationMessageDto,
  Sender,
  useConversation,
} from "src/api/conversational_analytics";
import { ConversationActionsMenu } from "src/components/ConversationalAnalytics/ConversationActionsMenu";
import { Flex, Icon, Text } from "src/components/design-system";
import { MessageHoverFooter } from "./MessageHoverFooter";
import styles from "./ExistingConversationPanel.module.css";
import { Fragment, useEffect } from "react";
import { AgentTrace } from "src/components/ConversationalAnalytics/AgentTrace";
import { StreamingAgentTrace } from "src/components/ConversationalAnalytics/StreamingAgentTrace";
import {
  useConversationStream,
  useStartStream,
} from "src/components/ConversationalAnalytics/ConversationStreamContext";

type MessageProps = {
  connectionId: string;
  message: ConversationMessageDto;
};

const Message = ({ connectionId, message }: MessageProps) => {
  const render = () => {
    if (message.sender == Sender.Human) {
      const text = [...message.content]
        .sort((a, b) => a.sequence_number - b.sequence_number)
        .map((c) => c.content)
        .join("\n");
      return (
        <Fragment>
          <Flex
            backgroundColor="grey"
            borderRadius="lg"
            paddingX="md"
            paddingY="md"
          >
            {text}
          </Flex>
          <MessageHoverFooter
            createdAt={message.created_at}
            textToCopy={text}
            footerClassName={styles.messageFooter}
          />
        </Fragment>
      );
    }
    if (message.sender == Sender.Assistant) {
      return (
        <AgentTrace connectionId={connectionId} contents={message.content} />
      );
    }
  };
  return (
    <Flex direction="column" className={styles.messageWrapper}>
      {render()}
    </Flex>
  );
};

export const ExistingConversationPanel = () => {
  const { id } = useParams<{ id: string }>();
  const conversationId = id ?? "";
  const conversationQuery = useConversation(conversationId);
  const startStream = useStartStream();
  const { events: streamingEvents, isStreaming } =
    useConversationStream(conversationId);

  useEffect(() => {
    if (!conversationId || !conversationQuery.isSuccess) return;
    if (!conversationQuery.data?.should_execute_completion) return;
    startStream(conversationId);
  }, [
    conversationId,
    conversationQuery.isSuccess,
    conversationQuery.data?.should_execute_completion,
    startStream,
  ]);

  const connectionId = conversationQuery.data?.conversation.connection_id ?? "";

  const render = () => {
    return conversationQuery.data?.messages.map(
      (message: ConversationMessageDto) => (
        <Message
          key={message.id}
          connectionId={connectionId}
          message={message}
        />
      ),
    );
  };

  const conversation = conversationQuery.data?.conversation;

  return (
    <Flex direction="column" paddingTop="lg" overflow="scrollY">
      {conversation && (
        <Flex paddingBottom="md" alignItems="center" width="100%">
          <ConversationActionsMenu conversation={conversation}>
            <button
              type="button"
              className={styles.conversationHeaderTrigger}
              aria-label="Conversation actions"
            >
              <Flex flex="grow" alignItems="center" sx={{ minWidth: 0 }}>
                <Text truncate fontSize="lg" color="darkGrey">
                  {conversation.name}
                </Text>
              </Flex>
              <Icon type="chevron_down" color="grey" size="md" />
            </button>
          </ConversationActionsMenu>
        </Flex>
      )}
      <Flex direction="column" className={styles.messageColumn}>
        {render()}
        {(isStreaming || streamingEvents.length > 0) && (
          <Flex direction="column" className={styles.messageWrapper}>
            <StreamingAgentTrace
              events={streamingEvents}
              isStreaming={isStreaming}
            />
          </Flex>
        )}
      </Flex>
    </Flex>
  );
};
