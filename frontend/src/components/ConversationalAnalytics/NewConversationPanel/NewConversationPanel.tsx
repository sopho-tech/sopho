import { useState } from "react";
import { generatePath, useNavigate } from "react-router";
import { Flex, Heading, Select } from "src/components/design-system";
import { useCreateConversation } from "src/api/conversational_analytics/queries";
import { useConnections } from "src/api/connection/queries";
import { useAiConfiguration } from "src/api/ai_configuration";
import { APP_ROUTES } from "src/constants/app_routes";
import { MessageComposer } from "src/components/ConversationalAnalytics/MessageComposer";
import styles from "./NewConversationPanel.module.css";

export function NewConversationPanel() {
  const navigate = useNavigate();
  const { mutate: createConversation, isPending } = useCreateConversation();
  const { data: connections } = useConnections();

  const [selectedConnectionId, setSelectedConnectionId] = useState("");
  const { data: aiConfiguration } = useAiConfiguration();
  const isAiReady = aiConfiguration?.status === "live";
  const disabled = !isAiReady || !selectedConnectionId || isPending;

  const aiDisabledTooltip =
    "Configure a working AI provider in Settings → AI Configurations to start new conversations.";
  const connectionDisabledTooltip = "Select a connection to start.";
  const disabledTooltip = !isAiReady
    ? aiDisabledTooltip
    : !selectedConnectionId
      ? connectionDisabledTooltip
      : undefined;

  const handleSubmit = (user_message: string) => {
    if (!selectedConnectionId) return;
    createConversation(
      { connection_id: selectedConnectionId, user_message },
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
          placeholder="Hi, how can I help you today ?"
          onSubmit={handleSubmit}
          disabled={disabled}
          disabledTooltip={disabledTooltip}
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
      </Flex>
    </Flex>
  );
}
