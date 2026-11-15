import { Sender, useConversation } from "src/api/conversational_analytics";
import { useAiConfiguration } from "src/api/ai_configuration";
import { useConversationStream } from "src/components/ConversationalAnalytics/ConversationStreamContext";

export type FollowUpGateReason =
  | "ai_not_ready"
  | "streaming"
  | "awaiting_response"
  | "loading"
  | "conversation_full";

export type FollowUpGate =
  | { canSend: true }
  | {
      canSend: false;
      reason: FollowUpGateReason;
      terminal: boolean;
      aiUnavailableMessage: string | null;
    };

export function useCanSendFollowUp(conversationId: string): FollowUpGate {
  const conversationQuery = useConversation(conversationId);
  const { data: aiConfiguration } = useAiConfiguration();
  const { isStreaming } = useConversationStream(conversationId);

  if (conversationQuery.isLoading || !conversationQuery.data) {
    return {
      canSend: false,
      reason: "loading",
      terminal: false,
      aiUnavailableMessage: null,
    };
  }
  if (conversationQuery.data.user_message_limit_reached) {
    return {
      canSend: false,
      reason: "conversation_full",
      terminal: true,
      aiUnavailableMessage: null,
    };
  }
  if (aiConfiguration?.is_ai_available !== true) {
    return {
      canSend: false,
      reason: "ai_not_ready",
      terminal: false,
      aiUnavailableMessage: aiConfiguration?.unavailable_message ?? null,
    };
  }
  if (isStreaming) {
    return {
      canSend: false,
      reason: "streaming",
      terminal: false,
      aiUnavailableMessage: null,
    };
  }
  const messages = conversationQuery.data.messages;
  const lastMessage = messages[messages.length - 1];
  if (lastMessage && lastMessage.sender === Sender.Human) {
    return {
      canSend: false,
      reason: "awaiting_response",
      terminal: false,
      aiUnavailableMessage: null,
    };
  }
  return { canSend: true };
}

export function getFollowUpDisabledTooltip(reason: FollowUpGateReason): string {
  switch (reason) {
    case "ai_not_ready":
      return "Configure AI provider in Settings";
    case "streaming":
      return "Wait for the current response to finish.";
    case "awaiting_response":
      return "Wait for the current response to finish.";
    case "loading":
      return "Loading conversation…";
    case "conversation_full":
      return "This conversation has reached its message limit.";
  }
}
