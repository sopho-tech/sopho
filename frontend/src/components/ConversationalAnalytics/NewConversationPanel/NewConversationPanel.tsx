import { useEffect, useRef, useState } from "react";
import { generatePath, useNavigate } from "react-router";
import { Flex, Heading, Select } from "src/components/design-system";
import { useCreateConversation } from "src/api/conversational_analytics/queries";
import { useConnections } from "src/api/connection/queries";
import { useAiConfiguration } from "src/api/ai_configuration";
import { APP_ROUTES } from "src/constants/app_routes";
import { UNAVAILABLE_CONNECTION_STATUS_LABEL } from "src/components/Connection/constants";
import { useTenantPath } from "src/utils/tenant";
import {
  MessageComposer,
  type MessageComposerHandle,
} from "src/components/ConversationalAnalytics/MessageComposer";
import type { MessageSegment } from "src/api/conversational_analytics/queries";
import { SuggestedQuestions } from "src/components/ConversationalAnalytics/NewConversationPanel/SuggestedQuestions";
import { SlashCommandSuggestionList } from "src/components/ConversationalAnalytics/SlashCommandSuggestionList";
import { SLASH_COMMANDS_FOR_FIRST_MESSAGE } from "src/constants/slash_commands";
import styles from "src/components/ConversationalAnalytics/NewConversationPanel/NewConversationPanel.module.css";

export function NewConversationPanel() {
  const navigate = useNavigate();
  const toTenantPath = useTenantPath();
  const { mutate: createConversation, isPending } = useCreateConversation();
  const { data: connections } = useConnections(null);

  const [selectedConnectionId, setSelectedConnectionId] = useState("");
  const composerRef = useRef<MessageComposerHandle>(null);

  useEffect(() => {
    if (selectedConnectionId) return;
    const firstAvailableConnection = connections?.find(
      (connection) => connection.is_source_type_available,
    );
    if (firstAvailableConnection) {
      setSelectedConnectionId(firstAvailableConnection.id);
    }
  }, [connections, selectedConnectionId]);

  const { data: aiConfiguration } = useAiConfiguration();
  const isAiReady = aiConfiguration?.is_ai_available === true;
  const disabled = !isAiReady || !selectedConnectionId || isPending;

  const aiDisabledTooltip =
    aiConfiguration?.unavailable_message ?? "Configure AI provider in Settings";
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
          void navigate(
            toTenantPath(
              generatePath(
                APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION,
                { id: data.id },
              ),
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
        <Heading accessibilityLevel={2} weight="normal" textAlign="center">
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
                {(connections ?? []).map((connection) => (
                  <Select.Item
                    key={connection.id}
                    value={connection.id}
                    disabled={!connection.is_source_type_available}
                    description={
                      connection.is_source_type_available
                        ? null
                        : UNAVAILABLE_CONNECTION_STATUS_LABEL
                    }
                  >
                    {connection.name}
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
