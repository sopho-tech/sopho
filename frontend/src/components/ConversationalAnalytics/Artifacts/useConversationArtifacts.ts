import { useMemo, useRef } from "react";
import { useConversation } from "src/api/conversational_analytics";
import { useConversationStream } from "src/components/ConversationalAnalytics/ConversationStreamContext";
import {
  extractArtifactsFromEvents,
  extractArtifactsFromMessages,
  type Artifact,
} from "src/components/ConversationalAnalytics/dto";

type ArtifactCache = {
  conversationId: string;
  artifacts: Map<string, Artifact>;
};

export function useConversationArtifacts(conversationId: string): Artifact[] {
  const { data } = useConversation(conversationId);
  const { events } = useConversationStream(conversationId);
  const cacheRef = useRef<ArtifactCache>({
    conversationId,
    artifacts: new Map(),
  });

  return useMemo(() => {
    const cache = cacheRef.current;
    if (cache.conversationId !== conversationId) {
      cache.conversationId = conversationId;
      cache.artifacts = new Map();
    }

    const seen = [
      ...extractArtifactsFromMessages(data?.messages ?? []),
      ...extractArtifactsFromEvents(events),
    ];
    for (const artifact of seen) {
      if (!cache.artifacts.has(artifact.id)) {
        cache.artifacts.set(artifact.id, artifact);
      }
    }

    return [...cache.artifacts.values()];
  }, [conversationId, data?.messages, events]);
}
