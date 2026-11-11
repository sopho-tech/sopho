import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import {
  AiConfiguration,
  TestAiConfigurationInput,
  TestAiConfigurationResult,
  UpsertAiConfigurationInput,
} from "src/api/ai_configuration/dto";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

export const aiConfigurationKeys = {
  all: (tenantId: string) => [tenantId, "ai_configuration"] as const,
  detail: (tenantId: string) =>
    [...aiConfigurationKeys.all(tenantId), "detail"] as const,
};

export const aiConfigurationApi = {
  get: async (tenantId: string): Promise<AiConfiguration> => {
    return (await ApiService.get<AiConfiguration>({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.AI_CONFIGURATION.GET),
      onlyBody: true,
    })) as AiConfiguration;
  },
  update: async (
    tenantId: string,
    payload: UpsertAiConfigurationInput,
  ): Promise<AiConfiguration> => {
    return (await ApiService.put<AiConfiguration>({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.AI_CONFIGURATION.UPDATE),
      data: payload,
      onlyBody: true,
      headers: { "Content-Type": "application/json" },
    })) as AiConfiguration;
  },
  remove: async (tenantId: string): Promise<void> => {
    await ApiService.delete({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.AI_CONFIGURATION.DELETE),
    });
  },
  test: async (
    tenantId: string,
    payload: TestAiConfigurationInput,
  ): Promise<TestAiConfigurationResult> => {
    return (await ApiService.post<TestAiConfigurationResult>({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.AI_CONFIGURATION.TEST),
      data: payload,
      onlyBody: true,
      headers: { "Content-Type": "application/json" },
    })) as TestAiConfigurationResult;
  },
};

export const useAiConfiguration = () => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: aiConfigurationKeys.detail(tenantId),
    queryFn: () => aiConfigurationApi.get(tenantId),
    staleTime: 30 * 1000,
  });
};

export const useUpdateAiConfiguration = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: (payload: UpsertAiConfigurationInput) =>
      aiConfigurationApi.update(tenantId, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: aiConfigurationKeys.detail(tenantId),
      });
    },
  });
};

export const useDeleteAiConfiguration = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: () => aiConfigurationApi.remove(tenantId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: aiConfigurationKeys.detail(tenantId),
      });
    },
  });
};

export const useTestAiConfiguration = () => {
  const tenantId = useTenantId();
  return useMutation({
    mutationFn: (payload: TestAiConfigurationInput) =>
      aiConfigurationApi.test(tenantId, payload),
  });
};
