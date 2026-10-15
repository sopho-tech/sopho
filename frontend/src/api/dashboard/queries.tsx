import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { DashboardDto } from "src/components/Dashboard/dto";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";
import { canvasKeys } from "src/api/canvas/queries";

export const dashboardKeys = {
  all: (tenantId: string) => [tenantId, "dashboards"] as const,
  details: (tenantId: string) =>
    [...dashboardKeys.all(tenantId), "detail"] as const,
  detail: (tenantId: string, id: string) =>
    [...dashboardKeys.details(tenantId), id] as const,
  byCanvas: (tenantId: string, canvasId: string) =>
    [...dashboardKeys.all(tenantId), "byCanvas", canvasId] as const,
};

const dashboardApi = {
  getDashboard: async (
    tenantId: string,
    dashboardId: string,
  ): Promise<DashboardDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.DASHBOARD.GET_BY_ID.replace(":id", dashboardId),
      ),
      onlyBody: true,
    });
    return response as DashboardDto;
  },

  getDashboardByCanvasId: async (
    tenantId: string,
    canvasId: string,
  ): Promise<DashboardDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.DASHBOARD.GET_BY_CANVAS_ID.replace(
          ":canvas_id",
          canvasId,
        ),
      ),
      onlyBody: true,
    });
    return response as DashboardDto;
  },

  updateDashboard: async (
    tenantId: string,
    {
      dashboardId,
      payload,
    }: {
      dashboardId: string;
      payload: Partial<DashboardDto>;
    },
  ): Promise<DashboardDto> => {
    const response = await ApiService.put({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.DASHBOARD.UPDATE.replace(":id", dashboardId),
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as DashboardDto;
  },
};

export const useDashboard = (dashboardId: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: dashboardKeys.detail(tenantId, dashboardId),
    queryFn: () => dashboardApi.getDashboard(tenantId, dashboardId),
    enabled: !!dashboardId,
  });
};

export const useDashboardByCanvasId = (canvasId: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: dashboardKeys.byCanvas(tenantId, canvasId),
    queryFn: () => dashboardApi.getDashboardByCanvasId(tenantId, canvasId),
    enabled: !!canvasId,
  });
};

export const useUpdateDashboard = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (variables: {
      dashboardId: string;
      payload: Partial<DashboardDto>;
    }) => dashboardApi.updateDashboard(tenantId, variables),
    onSuccess: (data, { dashboardId }) => {
      queryClient.setQueryData(
        dashboardKeys.detail(tenantId, dashboardId),
        data,
      );
      void queryClient.invalidateQueries({
        queryKey: dashboardKeys.all(tenantId),
      });
      void queryClient.invalidateQueries({
        queryKey: canvasKeys.details(tenantId),
      });
    },
    onError: (error) => {
      console.error("Failed to update dashboard:", error);
    },
  });
};
