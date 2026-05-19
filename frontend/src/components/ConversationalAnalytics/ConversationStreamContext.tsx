import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  type ReactNode,
} from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useSyncExternalStore } from "react";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { conversationKeys } from "src/api/conversational_analytics/queries";
import { useSuggestConversationName } from "src/api/conversational_analytics";
import { streamSse } from "src/utils/fetch_sse";
import { AgentEventName, type AgentEvent } from "./dto";

const TERMINAL_EVENTS = new Set<string>([
  AgentEventName.Completed,
  AgentEventName.Error,
  AgentEventName.AwaitingClarification,
  AgentEventName.Rejected,
]);

function isTerminalEvent(eventName: string): boolean {
  return TERMINAL_EVENTS.has(eventName);
}

type StreamEntry = {
  events: AgentEvent[];
  isStreaming: boolean;
};

const EMPTY_STREAM: StreamEntry = { events: [], isStreaming: false };

type StreamStore = {
  streams: Map<string, StreamEntry>;
  controllers: Map<string, AbortController>;
  nameSuggested: Set<string>;
  listeners: Set<() => void>;
};

function createStreamStore(): StreamStore {
  return {
    streams: new Map(),
    controllers: new Map(),
    nameSuggested: new Set(),
    listeners: new Set(),
  };
}

function notify(store: StreamStore) {
  store.listeners.forEach((l) => l());
}

type ConversationStreamContextValue = {
  getStream: (conversationId: string) => StreamEntry;
  startStream: (conversationId: string) => void;
  subscribe: (listener: () => void) => () => void;
};

const ConversationStreamContext =
  createContext<ConversationStreamContextValue | null>(null);

export function ConversationStreamProvider({
  children,
}: {
  children: ReactNode;
}) {
  const storeRef = useRef<StreamStore>(createStreamStore());
  const queryClient = useQueryClient();
  const suggestName = useSuggestConversationName();
  const suggestNameRef = useRef(suggestName);
  suggestNameRef.current = suggestName;

  const subscribe = useCallback((listener: () => void) => {
    const store = storeRef.current;
    store.listeners.add(listener);
    return () => {
      store.listeners.delete(listener);
    };
  }, []);

  const getStream = useCallback((conversationId: string): StreamEntry => {
    return storeRef.current.streams.get(conversationId) ?? EMPTY_STREAM;
  }, []);

  const startStream = useCallback(
    (conversationId: string) => {
      const store = storeRef.current;

      if (store.controllers.has(conversationId)) return;

      if (!store.nameSuggested.has(conversationId)) {
        store.nameSuggested.add(conversationId);
        suggestNameRef.current.mutate(conversationId);
      }

      const controller = new AbortController();
      store.controllers.set(conversationId, controller);
      store.streams.set(conversationId, { events: [], isStreaming: true });
      notify(store);

      const endpoint =
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.COMPLETION.replace(
          ":conversation_id",
          conversationId,
        );

      const run = async () => {
        try {
          await streamSse(
            endpoint,
            {
              signal: controller.signal,
              credentials: "include",
              method: "POST",
            },
            {
              onMessage: (data) => {
                try {
                  const parsed = JSON.parse(data) as AgentEvent;
                  if (isTerminalEvent(parsed.event_name)) {
                    store.controllers.delete(conversationId);
                    store.streams.delete(conversationId);
                    notify(store);
                    controller.abort();
                    queryClient.invalidateQueries({
                      queryKey: conversationKeys.detail(conversationId),
                    });
                    return;
                  }
                  const prev = store.streams.get(conversationId)?.events ?? [];
                  store.streams.set(conversationId, {
                    events: [...prev, parsed],
                    isStreaming: true,
                  });
                  notify(store);
                } catch {
                  /* ignore unparseable frames */
                }
              },
            },
          );
        } catch (e) {
          if (e instanceof DOMException && e.name === "AbortError") return;
          console.error(e);
        } finally {
          store.controllers.delete(conversationId);
          if (store.streams.has(conversationId)) {
            store.streams.delete(conversationId);
            notify(store);
          }
        }
      };

      void run();
    },
    [queryClient],
  );

  useEffect(() => {
    const store = storeRef.current;
    return () => {
      store.controllers.forEach((c) => c.abort());
      store.controllers.clear();
    };
  }, []);

  const value: ConversationStreamContextValue = {
    getStream,
    startStream,
    subscribe,
  };

  return (
    <ConversationStreamContext.Provider value={value}>
      {children}
    </ConversationStreamContext.Provider>
  );
}

export function useConversationStream(conversationId: string): StreamEntry {
  const ctx = useContext(ConversationStreamContext);
  if (!ctx) {
    throw new Error(
      "useConversationStream must be used within ConversationStreamProvider",
    );
  }

  const { subscribe, getStream } = ctx;
  const getSnapshot = useCallback(
    () => getStream(conversationId),
    [getStream, conversationId],
  );

  return useSyncExternalStore(subscribe, getSnapshot);
}

export function useStartStream() {
  const ctx = useContext(ConversationStreamContext);
  if (!ctx) {
    throw new Error(
      "useStartStream must be used within ConversationStreamProvider",
    );
  }
  return ctx.startStream;
}
