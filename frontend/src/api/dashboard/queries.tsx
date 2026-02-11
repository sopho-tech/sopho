import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { DashboardDto } from "src/components/Dashboard/dto";

export const dashboardKeys = {
  all: ["dashboards"] as const,
  details: () => [...dashboardKeys.all, "detail"] as const,
  detail: (id: string) => [...dashboardKeys.details(), id] as const,
  byCanvas: (canvasId: string) =>
    [...dashboardKeys.all, "byCanvas", canvasId] as const,
};

const dashboardApi = {
  getDashboard: async (dashboardId: string): Promise<DashboardDto> => {
    const response = await ApiService.get({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.DASHBOARD.GET_BY_ID?.replace(":id", dashboardId) || `/dashboard/${dashboardId}`}`,
      onlyBody: true,
    });
    return response as DashboardDto;
  },

  getDashboardByCanvasId: async (canvasId: string): Promise<DashboardDto> => {
    const response = await ApiService.get({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.DASHBOARD.GET_BY_CANVAS_ID?.replace(":canvas_id", canvasId) || `/dashboard/canvas/${canvasId}`}`,
      onlyBody: true,
    });
    return response as DashboardDto;
  },

  updateDashboard: async ({
    dashboardId,
    payload,
  }: {
    dashboardId: string;
    payload: Partial<DashboardDto>;
  }): Promise<DashboardDto> => {
    const response = await ApiService.put({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.DASHBOARD.UPDATE.replace(":id", dashboardId)}`,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as DashboardDto;
  },
};

export const useDashboard = (dashboardId: string) => {
  return useQuery({
    queryKey: dashboardKeys.detail(dashboardId),
    queryFn: () => dashboardApi.getDashboard(dashboardId),
    enabled: !!dashboardId,
  });
};

export const useDashboardByCanvasId = (canvasId: string) => {
  return useQuery({
    queryKey: dashboardKeys.byCanvas(canvasId),
    queryFn: () => dashboardApi.getDashboardByCanvasId(canvasId),
    enabled: !!canvasId,
  });
};

export const useUpdateDashboard = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: dashboardApi.updateDashboard,
    onSuccess: (data, { dashboardId }) => {
      queryClient.setQueryData(dashboardKeys.detail(dashboardId), data);
      queryClient.invalidateQueries({
        queryKey: dashboardKeys.all,
      });
    },
    onError: (error) => {
      console.error("Failed to update dashboard:", error);
    },
  });
};
