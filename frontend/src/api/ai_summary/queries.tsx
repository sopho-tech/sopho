import {
  queryOptions,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { ApiError } from "src/api/dto";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { ApiService } from "src/utils/api_client";

const GENERATING_POLL_INTERVAL_MS = 3000;

export type SummaryStatus = "GENERATING" | "READY" | "FAILED";

export type SummaryEntityType = "DASHBOARD" | "CHART_CELL";

export type AiSummaryDto = {
  entity_type: SummaryEntityType;
  entity_id: string;
  status: SummaryStatus;
  summary_text: string | null;
  error_message: string | null;
  summarized_entity_count: number | null;
  user_prompt: string | null;
  user_prompt_used: string | null;
  is_prompt_stale: boolean;
  generated_at: string | null;
  requested_at: string | null;
};

export type DashboardSummariesDto = {
  dashboard: AiSummaryDto | null;
  charts: AiSummaryDto[];
};

export const aiSummaryKeys = {
  all: ["ai_summaries"] as const,
  dashboard: (dashboardId: string) =>
    [...aiSummaryKeys.all, "dashboard", dashboardId] as const,
};

async function generate(url: string): Promise<AiSummaryDto | null> {
  try {
    const response = await ApiService.post<AiSummaryDto>({ url });
    return response as AiSummaryDto;
  } catch (error) {
    if (error instanceof ApiError && error.status === 409) {
      return null;
    }
    throw error;
  }
}

const aiSummaryApi = {
  getForDashboard: async (
    dashboardId: string
  ): Promise<DashboardSummariesDto> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.AI_SUMMARY.GET_BY_DASHBOARD.replace(
        ":dashboard_id",
        dashboardId
      ),
      onlyBody: true,
    });
    return response as DashboardSummariesDto;
  },

  generateForDashboard: (dashboardId: string) =>
    generate(
      API_ENDPOINTS.AI_SUMMARY.GENERATE_DASHBOARD.replace(
        ":dashboard_id",
        dashboardId
      )
    ),

  generateForChart: (cellId: string) =>
    generate(
      API_ENDPOINTS.AI_SUMMARY.GENERATE_CHART.replace(":cell_id", cellId)
    ),

  setDashboardPrompt: async (
    dashboardId: string,
    userPrompt: string | null
  ): Promise<AiSummaryDto> =>
    (await ApiService.put<AiSummaryDto>({
      url: API_ENDPOINTS.AI_SUMMARY.SET_DASHBOARD_PROMPT.replace(
        ":dashboard_id",
        dashboardId
      ),
      data: { user_prompt: userPrompt },
      headers: { "Content-Type": "application/json" },
    })) as AiSummaryDto,

  setChartPrompt: async (
    cellId: string,
    userPrompt: string | null
  ): Promise<AiSummaryDto> =>
    (await ApiService.put<AiSummaryDto>({
      url: API_ENDPOINTS.AI_SUMMARY.SET_CHART_PROMPT.replace(":cell_id", cellId),
      data: { user_prompt: userPrompt },
      headers: { "Content-Type": "application/json" },
    })) as AiSummaryDto,

  generateForAllCharts: async (dashboardId: string): Promise<AiSummaryDto[]> => {
    const response = await ApiService.post<AiSummaryDto[]>({
      url: API_ENDPOINTS.AI_SUMMARY.GENERATE_ALL_CHARTS.replace(
        ":dashboard_id",
        dashboardId
      ),
    });
    return response as AiSummaryDto[];
  },
};

function hasGeneratingSummary(data: DashboardSummariesDto | undefined) {
  if (!data) {
    return false;
  }
  return (
    data.dashboard?.status === "GENERATING" ||
    data.charts.some((chart) => chart.status === "GENERATING")
  );
}

const dashboardSummariesOptions = (dashboardId: string) =>
  queryOptions({
    queryKey: aiSummaryKeys.dashboard(dashboardId),
    queryFn: () => aiSummaryApi.getForDashboard(dashboardId),
    enabled: !!dashboardId,
    refetchInterval: (query) =>
      hasGeneratingSummary(query.state.data)
        ? GENERATING_POLL_INTERVAL_MS
        : false,
  });

export const useDashboardSummary = (dashboardId: string) =>
  useQuery({
    ...dashboardSummariesOptions(dashboardId),
    select: (data) => data.dashboard,
  });

export const useAnyChartSummaryGenerating = (dashboardId: string) =>
  useQuery({
    ...dashboardSummariesOptions(dashboardId),
    select: (data) =>
      data.charts.some((chart) => chart.status === "GENERATING"),
  });

export const useChartSummary = (dashboardId: string, cellId: string) =>
  useQuery({
    ...dashboardSummariesOptions(dashboardId),
    select: (data) =>
      data.charts.find((chart) => chart.entity_id === cellId) ?? null,
  });

export const useGenerateDashboardSummary = (dashboardId: string) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => aiSummaryApi.generateForDashboard(dashboardId),
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: aiSummaryKeys.dashboard(dashboardId),
      });
    },
  });
};

export const useGenerateChartSummary = (dashboardId: string) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (cellId: string) => aiSummaryApi.generateForChart(cellId),
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: aiSummaryKeys.dashboard(dashboardId),
      });
    },
  });
};

export const useUpdateDashboardPrompt = (dashboardId: string) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (userPrompt: string | null) =>
      aiSummaryApi.setDashboardPrompt(dashboardId, userPrompt),
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: aiSummaryKeys.dashboard(dashboardId),
      });
    },
  });
};

export const useUpdateChartPrompt = (dashboardId: string, cellId: string) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (userPrompt: string | null) =>
      aiSummaryApi.setChartPrompt(cellId, userPrompt),
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: aiSummaryKeys.dashboard(dashboardId),
      });
    },
  });
};

export const useGenerateAllChartSummaries = (dashboardId: string) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => aiSummaryApi.generateForAllCharts(dashboardId),
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: aiSummaryKeys.dashboard(dashboardId),
      });
    },
  });
};
