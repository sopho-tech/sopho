import { useQuery } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

export type SuggestedQuestionDto = {
  id: string;
  question_text: string;
  generated_at: string;
};

export const suggestedQuestionKeys = {
  all: (tenantId: string) => [tenantId, "suggested_questions"] as const,
  forConnection: (tenantId: string, id: string) =>
    [...suggestedQuestionKeys.all(tenantId), id] as const,
};

const suggestedQuestionApi = {
  getForConnection: async (
    tenantId: string,
    connectionId: string,
  ): Promise<SuggestedQuestionDto[]> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.SUGGESTED_QUESTION.GET_BY_CONNECTION.replace(
          ":connection_id",
          connectionId,
        ),
      ),
      onlyBody: true,
    });
    return response as SuggestedQuestionDto[];
  },
};

export const useSuggestedQuestions = (connectionId: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: suggestedQuestionKeys.forConnection(tenantId, connectionId),
    queryFn: () =>
      suggestedQuestionApi.getForConnection(tenantId, connectionId),
    enabled: !!connectionId,
    staleTime: 1000 * 60 * 60 * 24,
  });
};
