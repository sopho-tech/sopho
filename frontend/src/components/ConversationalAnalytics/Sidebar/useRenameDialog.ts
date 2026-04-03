import { useCallback, useEffect, useRef, useState } from "react";
import {
  ConversationDto,
  useUpdateConversation,
} from "src/api/conversational_analytics";

export function useRenameDialog() {
  const [renameTarget, setRenameTarget] = useState<ConversationDto | null>(
    null,
  );
  const [open, setOpen] = useState(false);
  const unmountTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const openTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const updateConversation = useUpdateConversation();

  useEffect(() => {
    return () => {
      if (unmountTimerRef.current) clearTimeout(unmountTimerRef.current);
      if (openTimerRef.current) clearTimeout(openTimerRef.current);
    };
  }, []);

  const openRename = useCallback((conversation: ConversationDto) => {
    if (unmountTimerRef.current) {
      clearTimeout(unmountTimerRef.current);
      unmountTimerRef.current = null;
    }
    if (openTimerRef.current) {
      clearTimeout(openTimerRef.current);
      openTimerRef.current = null;
    }
    setRenameTarget(conversation);
    setOpen(false);
    openTimerRef.current = setTimeout(() => {
      openTimerRef.current = null;
      setOpen(true);
    }, 0);
  }, []);

  const close = useCallback(() => {
    setOpen(false);
  }, []);

  const handleOpenChange = useCallback((nextOpen: boolean) => {
    if (!nextOpen) {
      setOpen(false);
      if (unmountTimerRef.current) clearTimeout(unmountTimerRef.current);
      unmountTimerRef.current = setTimeout(() => {
        unmountTimerRef.current = null;
        setRenameTarget(null);
      }, 300);
    }
  }, []);

  const submit = useCallback(
    (formData: FormData) => {
      if (!renameTarget) return;
      const name = (formData.get("name") as string)?.trim() ?? "";
      updateConversation.mutate(
        { ...renameTarget, name },
        { onSuccess: () => setOpen(false) },
      );
    },
    [renameTarget, updateConversation],
  );

  return { renameTarget, open, openRename, close, handleOpenChange, submit };
}
