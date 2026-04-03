import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router";
import {
  ConversationDto,
  useDeleteConversation,
} from "src/api/conversational_analytics";
import { APP_ROUTES } from "src/constants/app_routes";

export function useDeleteConversationDialog(conversationId: string) {
  const navigate = useNavigate();
  const { id: routeConversationId } = useParams<{ id: string }>();
  const [deleteTarget, setDeleteTarget] = useState<ConversationDto | null>(
    null,
  );
  const [open, setOpen] = useState(false);
  const unmountTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const openTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const deleteConversation = useDeleteConversation();

  useEffect(() => {
    return () => {
      if (unmountTimerRef.current) clearTimeout(unmountTimerRef.current);
      if (openTimerRef.current) clearTimeout(openTimerRef.current);
    };
  }, []);

  const openDelete = useCallback((conversation: ConversationDto) => {
    if (unmountTimerRef.current) {
      clearTimeout(unmountTimerRef.current);
      unmountTimerRef.current = null;
    }
    if (openTimerRef.current) {
      clearTimeout(openTimerRef.current);
      openTimerRef.current = null;
    }
    setDeleteTarget(conversation);
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
        setDeleteTarget(null);
      }, 300);
    }
  }, []);

  const confirm = useCallback(() => {
    if (!deleteTarget) return;
    const id = deleteTarget.id;
    deleteConversation.mutate(id, {
      onSuccess: () => {
        if (routeConversationId === id && id === conversationId) {
          navigate(APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.INDEX);
        }
        setOpen(false);
        if (unmountTimerRef.current) clearTimeout(unmountTimerRef.current);
        unmountTimerRef.current = setTimeout(() => {
          unmountTimerRef.current = null;
          setDeleteTarget(null);
        }, 300);
      },
    });
  }, [
    deleteTarget,
    deleteConversation,
    routeConversationId,
    conversationId,
    navigate,
  ]);

  return {
    deleteTarget,
    open,
    openDelete,
    close,
    handleOpenChange,
    confirm,
    isPending: deleteConversation.isPending,
  };
}
