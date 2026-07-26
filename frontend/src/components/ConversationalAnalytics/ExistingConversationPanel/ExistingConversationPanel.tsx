import { useParams } from "react-router";
import {
  ConversationMessageDto,
  MessageStatus,
  Sender,
  useAppendUserMessage,
  useConversation,
} from "src/api/conversational_analytics";
import { ConversationActionsMenu } from "src/components/ConversationalAnalytics/ConversationActionsMenu";
import { Flex, Icon, Text } from "src/components/design-system";
import { MessageHoverFooter } from "./MessageHoverFooter";
import {
  getFollowUpDisabledTooltip,
  useCanSendFollowUp,
} from "./useCanSendFollowUp";
import styles from "./ExistingConversationPanel.module.css";
import { useEffect, useMemo, useRef } from "react";
import { AgentTrace } from "src/components/ConversationalAnalytics/AgentTrace";
import { StreamingAgentTrace } from "src/components/ConversationalAnalytics/StreamingAgentTrace";
import { MessageComposer } from "src/components/ConversationalAnalytics/MessageComposer";
import {
  useConversationStream,
  useStartStream,
} from "src/components/ConversationalAnalytics/ConversationStreamContext";
import type { MessageComposerHandle } from "src/components/ConversationalAnalytics/MessageComposer";
import type { MessageSegment } from "src/api/conversational_analytics/queries";
import { extractFollowUpQuestions } from "src/components/ConversationalAnalytics/dto";
import { SuggestedQuestionList } from "src/components/ConversationalAnalytics/SuggestedQuestionList";
import {
  MessageSegments,
  parseMessageSegments,
  segmentsToText,
} from "src/components/ConversationalAnalytics/MessageSegments";

type MessageProps = {
  connectionId: string;
  message: ConversationMessageDto;
};

const Message = ({ connectionId, message }: MessageProps) => {
  const render = () => {
    if (message.sender == Sender.Human) {
      const rawContent = [...message.content]
        .sort((a, b) => a.sequence_number - b.sequence_number)
        .map((c) => c.content)
        .join("\n");
      const segments = parseMessageSegments(rawContent);
      const text = segments ? segmentsToText(segments) : rawContent;
      return (
        <Flex direction="column" className={styles.messageBubbleGroup}>
          <Flex
            backgroundColor="grey"
            borderRadius="lg"
            paddingX="md"
            paddingY="md"
          >
            {segments ? <MessageSegments segments={segments} /> : rawContent}
          </Flex>
          <MessageHoverFooter
            createdAt={message.created_at}
            textToCopy={text}
            footerClassName={styles.messageFooter}
          />
        </Flex>
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
  const composerRef = useRef<MessageComposerHandle>(null);
  const { events: streamingEvents, isStreaming } =
    useConversationStream(conversationId);
  const appendUserMessage = useAppendUserMessage(conversationId);
  const gate = useCanSendFollowUp(conversationId);

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

  const handleSubmit = (segments: MessageSegment[]) => {
    if (!gate.canSend) return;
    appendUserMessage.mutate(
      { segments },
      {
        onSuccess: () => {
          startStream(conversationId);
        },
      },
    );
  };

  const composerDisabled = !gate.canSend || appendUserMessage.isPending;
  const composerDisabledTooltip = gate.canSend
    ? undefined
    : getFollowUpDisabledTooltip(gate.reason);

  const lastAssistantStatus = useMemo(() => {
    const all = conversationQuery.data?.messages ?? [];
    for (let i = all.length - 1; i >= 0; i--) {
      if (all[i].sender === Sender.Assistant) return all[i].status;
    }
    return null;
  }, [conversationQuery.data?.messages]);

  const composerPlaceholder =
    lastAssistantStatus === MessageStatus.AwaitingClarification
      ? "Answer the clarifying question…"
      : "Ask something else...";

  const messagesToRender = useMemo(() => {
    const all = conversationQuery.data?.messages ?? [];
    if (!isStreaming) return all;
    const last = all[all.length - 1];
    if (last?.sender === Sender.Assistant) return all.slice(0, -1);
    return all;
  }, [conversationQuery.data?.messages, isStreaming]);

  const followUpQuestions = useMemo(
    () => extractFollowUpQuestions(conversationQuery.data?.messages ?? []),
    [conversationQuery.data?.messages],
  );

  const renderMessages = () => {
    return messagesToRender.map((message: ConversationMessageDto) => (
      <Message key={message.id} connectionId={connectionId} message={message} />
    ));
  };

  const conversation = conversationQuery.data?.conversation;

  return (
    <Flex direction="column" height="100%" width="100%">
      <Flex direction="column" flex="grow" overflow="scrollY" paddingTop="lg">
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
          {renderMessages()}
          {(isStreaming || streamingEvents.length > 0) && (
            <Flex direction="column" className={styles.messageWrapper}>
              <StreamingAgentTrace
                events={streamingEvents}
                isStreaming={isStreaming}
              />
            </Flex>
          )}
          {gate.canSend && !isStreaming && followUpQuestions.length > 0 && (
            <SuggestedQuestionList
              questions={followUpQuestions.map((text) => ({ id: text, text }))}
              onPick={(text) => composerRef.current?.setText(text)}
            />
          )}
        </Flex>
      </Flex>
      <Flex
        direction="column"
        paddingTop="md"
        paddingBottom="md"
        sx={{ flexShrink: 0 }}
      >
        <MessageComposer
          ref={composerRef}
          placeholder={composerPlaceholder}
          onSubmit={handleSubmit}
          disabled={composerDisabled}
          disabledTooltip={composerDisabledTooltip}
        />
      </Flex>
    </Flex>
  );
};
