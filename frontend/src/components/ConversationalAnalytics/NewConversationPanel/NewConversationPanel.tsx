import { useEffect, useRef, useState } from "react";
import { generatePath, useNavigate } from "react-router";
import { Flex, Heading, Select } from "src/components/design-system";
import { useCreateConversation } from "src/api/conversational_analytics/queries";
import { useConnections } from "src/api/connection/queries";
import { useAiConfiguration } from "src/api/ai_configuration";
import { APP_ROUTES } from "src/constants/app_routes";
import {
  MessageComposer,
  type MessageComposerHandle,
} from "src/components/ConversationalAnalytics/MessageComposer";
import type { MessageSegment } from "src/api/conversational_analytics/queries";
import { SuggestedQuestions } from "src/components/ConversationalAnalytics/NewConversationPanel/SuggestedQuestions";
import { SlashCommandSuggestionList } from "src/components/ConversationalAnalytics/SlashCommandSuggestionList";
import { SLASH_COMMANDS_FOR_FIRST_MESSAGE } from "src/constants/slash_commands";
import styles from "./NewConversationPanel.module.css";

export function NewConversationPanel() {
  const navigate = useNavigate();
  const { mutate: createConversation, isPending } = useCreateConversation();
  const { data: connections } = useConnections();

  const [selectedConnectionId, setSelectedConnectionId] = useState("");
  const composerRef = useRef<MessageComposerHandle>(null);

  useEffect(() => {
    if (!selectedConnectionId && connections?.length) {
      setSelectedConnectionId(connections[0].id);
    }
  }, [connections, selectedConnectionId]);

  const { data: aiConfiguration } = useAiConfiguration();
  const isAiReady = aiConfiguration?.status === "live";
  const disabled = !isAiReady || !selectedConnectionId || isPending;

  const aiDisabledTooltip = "Configure AI provider in Settings";
  const connectionDisabledTooltip = "Select a connection to start.";
  const disabledTooltip = !isAiReady
    ? aiDisabledTooltip
    : !selectedConnectionId
      ? connectionDisabledTooltip
      : undefined;

  const handleSubmit = (segments: MessageSegment[]) => {
    if (!selectedConnectionId) return;
    createConversation(
      { connection_id: selectedConnectionId, segments },
      {
        onSuccess: (data) => {
          navigate(
            generatePath(
              APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION,
              { id: data.id },
            ),
          );
        },
      },
    );
  };

  return (
    <Flex direction="column" height="100%" width="100%" justifyContent="center">
      <Flex
        direction="column"
        width="100%"
        sx={{ maxWidth: "40rem", marginInline: "auto" }}
        gap="2xl"
      >
        <Heading accessbilityLevel={2} weight="normal" textAlign="center">
          Let's data
        </Heading>
        <MessageComposer
          ref={composerRef}
          placeholder={
            SLASH_COMMANDS_FOR_FIRST_MESSAGE.length > 0
              ? "Ask anything, or type / for commands"
              : "Ask anything about your data"
          }
          onSubmit={handleSubmit}
          disabled={disabled}
          disabledTooltip={disabledTooltip}
          availableCommands={SLASH_COMMANDS_FOR_FIRST_MESSAGE}
          slotLeft={
            <Select
              value={selectedConnectionId}
              onValueChange={setSelectedConnectionId}
            >
              <Select.Trigger
                placeholder="Connection"
                className={styles.connectionSelect}
              />
              <Select.Content>
                {(connections ?? []).map((c) => (
                  <Select.Item key={c.id} value={c.id}>
                    {c.name}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select>
          }
        />
        <SlashCommandSuggestionList
          commands={SLASH_COMMANDS_FOR_FIRST_MESSAGE}
          onPick={(commandName) =>
            composerRef.current?.insertCommand(commandName)
          }
        />
        <SuggestedQuestions
          connectionId={selectedConnectionId}
          onPick={(text) => composerRef.current?.setText(text)}
        />
      </Flex>
    </Flex>
  );
}
