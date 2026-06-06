import { useQuery } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";

export type SuggestedQuestionDto = {
  id: string;
  question_text: string;
  generated_at: string;
};

export const suggestedQuestionKeys = {
  all: ["suggested_questions"] as const,
  forConnection: (id: string) =>
    [...suggestedQuestionKeys.all, id] as const,
};

const suggestedQuestionApi = {
  getForConnection: async (
    connectionId: string,
  ): Promise<SuggestedQuestionDto[]> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.SUGGESTED_QUESTION.GET_BY_CONNECTION.replace(
        ":connection_id",
        connectionId,
      ),
      onlyBody: true,
    });
    return response as SuggestedQuestionDto[];
  },
};

export const useSuggestedQuestions = (connectionId: string) =>
  useQuery({
    queryKey: suggestedQuestionKeys.forConnection(connectionId),
    queryFn: () => suggestedQuestionApi.getForConnection(connectionId),
    enabled: !!connectionId,
    staleTime: 1000 * 60 * 60 * 24,
  });
