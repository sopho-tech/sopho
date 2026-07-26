import { useCallback, useState } from "react";

export function useConversationSelection() {
  const [isSelectMode, setIsSelectMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const clear = useCallback(() => setSelectedIds(new Set()), []);

  const exitSelectMode = useCallback(() => {
    setIsSelectMode(false);
    setSelectedIds(new Set());
  }, []);

  const enterSelectMode = useCallback((initialId?: string) => {
    setIsSelectMode(true);
    setSelectedIds(initialId ? new Set([initialId]) : new Set());
  }, []);

  const toggle = useCallback((conversationId: string) => {
    setSelectedIds((previous) => {
      const next = new Set(previous);
      if (next.has(conversationId)) {
        next.delete(conversationId);
      } else {
        next.add(conversationId);
      }
      return next;
    });
  }, []);

  const selectAll = useCallback((conversationIds: string[]) => {
    setSelectedIds(new Set(conversationIds));
  }, []);

  return {
    isSelectMode,
    selectedIds,
    enterSelectMode,
    exitSelectMode,
    toggle,
    selectAll,
    clear,
  };
}
