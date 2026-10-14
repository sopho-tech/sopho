import {
  keepPreviousData,
  QueryClient,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { ApiError } from "src/api/dto";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import {
  CreateConversationDto,
  ConversationDto,
  ConversationWithMessagesDto,
  ListConversationsParams,
  PaginatedConversationsDto,
} from "src/components/ConversationalAnalytics/dto";
import type { AppendUserMessageDto } from "src/components/ConversationalAnalytics/dto";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

export type {
  ConversationMessageContentDto,
  CreateConversationDto,
  ConversationDto,
  ConversationListItemDto,
  ConversationMessageDto,
  ConversationWithMessagesDto,
  AppendUserMessageDto,
  ListConversationsParams,
  PaginatedConversationsDto,
  MessageSegment,
} from "src/components/ConversationalAnalytics/dto";

export {
  MessageStatus,
  Sender,
} from "src/components/ConversationalAnalytics/dto";

export const conversationKeys = {
  all: (tenantId: string) => [tenantId, "conversations"] as const,
  lists: (tenantId: string) =>
    [...conversationKeys.all(tenantId), "list"] as const,
  list: (tenantId: string, params: ListConversationsParams) =>
    [...conversationKeys.lists(tenantId), params] as const,
  details: (tenantId: string) =>
    [...conversationKeys.all(tenantId), "detail"] as const,
  detail: (tenantId: string, id: string) =>
    [...conversationKeys.details(tenantId), id] as const,
};

function buildListConversationsUrl(
  tenantId: string,
  { page, pageSize, search }: ListConversationsParams,
): string {
  const searchParams = new URLSearchParams();
  if (page !== undefined) searchParams.set("page", String(page));
  if (pageSize !== undefined) searchParams.set("page_size", String(pageSize));
  if (search?.trim()) searchParams.set("search", search.trim());
  const query = searchParams.toString();
  const base = tenantApiUrl(
    tenantId,
    API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.GET_ALL,
  );
  return query ? `${base}?${query}` : base;
}

const conversationalAnalyticsApi = {
  listConversations: async (
    tenantId: string,
    params: ListConversationsParams,
  ): Promise<PaginatedConversationsDto> => {
    const response = await ApiService.get({
      url: buildListConversationsUrl(tenantId, params),
      onlyBody: true,
    });
    return response as PaginatedConversationsDto;
  },

  bulkDeleteConversations: async (
    tenantId: string,
    conversationIds: string[],
  ): Promise<void> => {
    await ApiService.post({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.BULK_DELETE,
      ),
      data: { conversation_ids: conversationIds },
      headers: { "Content-Type": "application/json" },
      onlyBody: false,
    });
  },

  getConversation: async (
    tenantId: string,
    conversationId: string,
  ): Promise<ConversationWithMessagesDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.GET_BY_ID.replace(
          ":conversation_id",
          conversationId,
        ),
      ),
      onlyBody: true,
    });
    return response as ConversationWithMessagesDto;
  },

  createConversation: async (
    tenantId: string,
    payload: CreateConversationDto,
  ): Promise<ConversationDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.CREATE,
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConversationDto;
  },

  updateConversation: async (
    tenantId: string,
    payload: ConversationDto,
  ): Promise<ConversationDto> => {
    const response = await ApiService.put({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.UPDATE.replace(
          ":conversation_id",
          payload.id,
        ),
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConversationDto;
  },

  suggestConversationName: async (
    tenantId: string,
    conversationId: string,
  ): Promise<ConversationDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.SUGGEST_NAME.replace(
          ":conversation_id",
          conversationId,
        ),
      ),
    });
    return response as ConversationDto;
  },

  appendUserMessage: async (
    tenantId: string,
    conversationId: string,
    payload: AppendUserMessageDto,
  ): Promise<ConversationWithMessagesDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.APPEND_MESSAGE.replace(
          ":conversation_id",
          conversationId,
        ),
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConversationWithMessagesDto;
  },

  deleteConversation: async (
    tenantId: string,
    conversationId: string,
  ): Promise<void> => {
    await ApiService.delete({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONVERSATIONAL_ANALYTICS.DELETE.replace(
          ":conversation_id",
          conversationId,
        ),
      ),
    });
  },
};

export const useConversations = (params: ListConversationsParams = {}) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: conversationKeys.list(tenantId, params),
    queryFn: () =>
      conversationalAnalyticsApi.listConversations(tenantId, params),
    placeholderData: keepPreviousData,
  });
};

const isClientError = (error: unknown): boolean =>
  error instanceof ApiError && error.status >= 400 && error.status < 500;

export const isConversationNotFoundError = (error: unknown): boolean =>
  error instanceof ApiError && error.status === 404;

export const useConversation = (conversationId: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: conversationKeys.detail(tenantId, conversationId),
    queryFn: () =>
      conversationalAnalyticsApi.getConversation(tenantId, conversationId),
    enabled: !!conversationId,
    retry: (failureCount, error) => !isClientError(error) && failureCount < 3,
  });
};

export const useCreateConversation = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (payload: CreateConversationDto) =>
      conversationalAnalyticsApi.createConversation(tenantId, payload),
    onSuccess: (data) => {
      queryClient.invalidateQueries({
        queryKey: conversationKeys.detail(tenantId, data.id),
      });
      queryClient.invalidateQueries({
        queryKey: conversationKeys.lists(tenantId),
      });
    },
  });
};

function patchConversationCaches(
  queryClient: QueryClient,
  tenantId: string,
  data: ConversationDto,
) {
  queryClient.setQueryData<ConversationWithMessagesDto>(
    conversationKeys.detail(tenantId, data.id),
    (old) => (old ? { ...old, conversation: data } : undefined),
  );
  queryClient.setQueriesData<PaginatedConversationsDto>(
    { queryKey: conversationKeys.lists(tenantId) },
    (old) =>
      old && {
        ...old,
        items: old.items.map((item) =>
          item.id === data.id ? { ...item, ...data } : item,
        ),
      },
  );
}

export const useSuggestConversationName = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (conversationId: string) =>
      conversationalAnalyticsApi.suggestConversationName(
        tenantId,
        conversationId,
      ),
    onSuccess: (data) => {
      patchConversationCaches(queryClient, tenantId, data);
    },
  });
};

export const useUpdateConversation = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (payload: ConversationDto) =>
      conversationalAnalyticsApi.updateConversation(tenantId, payload),
    onSuccess: (data) => {
      patchConversationCaches(queryClient, tenantId, data);
    },
  });
};

export const useDeleteConversation = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (conversationId: string) =>
      conversationalAnalyticsApi.deleteConversation(tenantId, conversationId),
    onSuccess: (_, conversationId) => {
      queryClient.removeQueries({
        queryKey: conversationKeys.detail(tenantId, conversationId),
      });
      queryClient.invalidateQueries({
        queryKey: conversationKeys.lists(tenantId),
      });
    },
  });
};

export const useBulkDeleteConversations = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (conversationIds: string[]) =>
      conversationalAnalyticsApi.bulkDeleteConversations(
        tenantId,
        conversationIds,
      ),
    onSuccess: (_, conversationIds) => {
      conversationIds.forEach((conversationId) => {
        queryClient.removeQueries({
          queryKey: conversationKeys.detail(tenantId, conversationId),
        });
      });
      queryClient.invalidateQueries({
        queryKey: conversationKeys.lists(tenantId),
      });
    },
  });
};

export const useAppendUserMessage = (conversationId: string) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (payload: AppendUserMessageDto) =>
      conversationalAnalyticsApi.appendUserMessage(
        tenantId,
        conversationId,
        payload,
      ),
    onSuccess: (data) => {
      queryClient.setQueryData<ConversationWithMessagesDto>(
        conversationKeys.detail(tenantId, conversationId),
        data,
      );
      queryClient.invalidateQueries({
        queryKey: conversationKeys.lists(tenantId),
      });
    },
  });
};
