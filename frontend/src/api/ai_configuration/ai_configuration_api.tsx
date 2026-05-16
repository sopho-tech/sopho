import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import {
  AiConfiguration,
  TestAiConfigurationInput,
  TestAiConfigurationResult,
  UpsertAiConfigurationInput,
} from "src/api/ai_configuration/dto";

export const aiConfigurationKeys = {
  all: ["ai_configuration"] as const,
  detail: () => [...aiConfigurationKeys.all, "detail"] as const,
};

export const aiConfigurationApi = {
  get: async (): Promise<AiConfiguration> => {
    const url = API_ENDPOINTS.AI_CONFIGURATION.GET;
    return (await ApiService.get<AiConfiguration>({
      url,
      onlyBody: true,
    })) as AiConfiguration;
  },
  update: async (
    payload: UpsertAiConfigurationInput,
  ): Promise<AiConfiguration> => {
    const url = API_ENDPOINTS.AI_CONFIGURATION.UPDATE;
    return (await ApiService.put<AiConfiguration>({
      url,
      data: payload,
      onlyBody: true,
      headers: { "Content-Type": "application/json" },
    })) as AiConfiguration;
  },
  remove: async (): Promise<void> => {
    const url = API_ENDPOINTS.AI_CONFIGURATION.DELETE;
    const response = await fetch(url, {
      method: "DELETE",
      credentials: "include",
    });
    if (!response.ok) {
      throw new Error(
        `Failed to delete AI configuration: ${response.statusText}`,
      );
    }
  },
  test: async (
    payload: TestAiConfigurationInput,
  ): Promise<TestAiConfigurationResult> => {
    const url = API_ENDPOINTS.AI_CONFIGURATION.TEST;
    return (await ApiService.post<TestAiConfigurationResult>({
      url,
      data: payload,
      onlyBody: true,
      headers: { "Content-Type": "application/json" },
    })) as TestAiConfigurationResult;
  },
};

export const useAiConfiguration = () =>
  useQuery({
    queryKey: aiConfigurationKeys.detail(),
    queryFn: aiConfigurationApi.get,
    staleTime: 30 * 1000,
  });

export const useUpdateAiConfiguration = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: aiConfigurationApi.update,
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: aiConfigurationKeys.detail(),
      });
    },
  });
};

export const useDeleteAiConfiguration = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: aiConfigurationApi.remove,
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: aiConfigurationKeys.detail(),
      });
    },
  });
};

export const useTestAiConfiguration = () =>
  useMutation({
    mutationFn: aiConfigurationApi.test,
  });
