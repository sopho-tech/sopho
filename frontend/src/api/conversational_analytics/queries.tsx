import {
  QueryClient,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import {
  CreateConversationDto,
  ConversationDto,
  ConversationWithMessagesDto,
} from "src/components/ConversationalAnalytics/dto";
import type { AppendUserMessageDto } from "src/components/ConversationalAnalytics/dto";

export type {
  ConversationMessageContentDto,
  CreateConversationDto,
  ConversationDto,
  ConversationMessageDto,
  ConversationWithMessagesDto,
  AppendUserMessageDto,
  MessageSegment,
} from "src/components/ConversationalAnalytics/dto";

export {
  MessageStatus,
  Sender,
} from "src/components/ConversationalAnalytics/dto";

export const conversationKeys = {
  all: ["conversations"] as const,
  lists: () => [...conversationKeys.all, "list"] as const,
  details: () => [...conversationKeys.all, "detail"] as const,
  detail: (id: string) => [...conversationKeys.details(), id] as const,
};

const conversationalAnalyticsApi = {
  getAllConversations: async (): Promise<ConversationDto[]> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.GET_ALL,
      onlyBody: true,
    });
    return response as ConversationDto[];
  },

  getConversation: async (
    conversationId: string,
  ): Promise<ConversationWithMessagesDto> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.GET_BY_ID.replace(
        ":conversation_id",
        conversationId,
      ),
      onlyBody: true,
    });
    return response as ConversationWithMessagesDto;
  },

  createConversation: async (
    payload: CreateConversationDto,
  ): Promise<ConversationDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.CREATE,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConversationDto;
  },

  updateConversation: async (
    payload: ConversationDto,
  ): Promise<ConversationDto> => {
    const response = await ApiService.put({
      url: API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.UPDATE.replace(
        ":conversation_id",
        payload.id,
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConversationDto;
  },

  suggestConversationName: async (
    conversationId: string,
  ): Promise<ConversationDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.SUGGEST_NAME.replace(
        ":conversation_id",
        conversationId,
      ),
    });
    return response as ConversationDto;
  },

  appendUserMessage: async (
    conversationId: string,
    payload: AppendUserMessageDto,
  ): Promise<ConversationWithMessagesDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.APPEND_MESSAGE.replace(
        ":conversation_id",
        conversationId,
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConversationWithMessagesDto;
  },

  deleteConversation: async (conversationId: string): Promise<void> => {
    const response = await fetch(
      API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.DELETE?.replace(
        ":conversation_id",
        conversationId,
      ),
      {
        method: "DELETE",
        credentials: "include",
      },
    );

    if (!response.ok) {
      throw new Error(`Failed to delete conversation: ${response.statusText}`);
    }
  },
};

export const useConversations = () => {
  return useQuery({
    queryKey: conversationKeys.lists(),
    queryFn: () => conversationalAnalyticsApi.getAllConversations(),
  });
};

export const useConversation = (conversationId: string) => {
  return useQuery({
    queryKey: conversationKeys.detail(conversationId),
    queryFn: () => conversationalAnalyticsApi.getConversation(conversationId),
    enabled: !!conversationId,
  });
};

export const useCreateConversation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: conversationalAnalyticsApi.createConversation,
    onSuccess: (data) => {
      queryClient.invalidateQueries({
        queryKey: conversationKeys.detail(data.id),
      });
      queryClient.invalidateQueries({
        queryKey: conversationKeys.lists(),
      });
    },
  });
};

function patchConversationCaches(
  queryClient: QueryClient,
  data: ConversationDto,
) {
  queryClient.setQueryData<ConversationWithMessagesDto>(
    conversationKeys.detail(data.id),
    (old) => (old ? { ...old, conversation: data } : undefined),
  );
  queryClient.setQueryData<ConversationDto[]>(conversationKeys.lists(), (old) =>
    old?.map((c) => (c.id === data.id ? data : c)),
  );
}

export const useSuggestConversationName = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: conversationalAnalyticsApi.suggestConversationName,
    onSuccess: (data) => {
      patchConversationCaches(queryClient, data);
    },
  });
};

export const useUpdateConversation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: conversationalAnalyticsApi.updateConversation,
    onSuccess: (data) => {
      patchConversationCaches(queryClient, data);
    },
  });
};

export const useDeleteConversation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: conversationalAnalyticsApi.deleteConversation,
    onSuccess: (_, conversationId) => {
      queryClient.removeQueries({
        queryKey: conversationKeys.detail(conversationId),
      });
      queryClient.invalidateQueries({
        queryKey: conversationKeys.lists(),
      });
    },
  });
};

export const useAppendUserMessage = (conversationId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: AppendUserMessageDto) =>
      conversationalAnalyticsApi.appendUserMessage(conversationId, payload),
    onSuccess: (data) => {
      queryClient.setQueryData<ConversationWithMessagesDto>(
        conversationKeys.detail(conversationId),
        data,
      );
      queryClient.invalidateQueries({
        queryKey: conversationKeys.lists(),
      });
    },
  });
};
