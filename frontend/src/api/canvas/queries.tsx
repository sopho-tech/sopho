import {
  useQuery,
  useMutation,
  useQueryClient,
  keepPreviousData,
} from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { useParams } from "react-router";
import { CanvasDto, NewCanvasDto } from "src/components/Canvases/dto";
import {
  PaginatedData,
  PaginatedDataResponse,
  ResourceCapabilities,
} from "src/api/dto";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

// Query keys for canvases
export const canvasKeys = {
  all: (tenantId: string) => [tenantId, "canvases"] as const,
  lists: (tenantId: string) => [...canvasKeys.all(tenantId), "list"] as const,
  listPaginated: (tenantId: string, page: number, pageSize: number) =>
    [...canvasKeys.lists(tenantId), { page, pageSize }] as const,
  details: (tenantId: string) =>
    [...canvasKeys.all(tenantId), "detail"] as const,
  detail: (tenantId: string, id: string) =>
    [...canvasKeys.details(tenantId), id] as const,
};

// API functions
const canvasApi = {
  getCanvas: async (tenantId: string, canvasId: string): Promise<CanvasDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CANVAS.GET_BY_ID.replace(":id", canvasId),
      ),
      onlyBody: true,
    });
    return response as CanvasDto;
  },

  getAllCanvases: async (
    tenantId: string,
    page?: number,
    pageSize?: number
  ): Promise<PaginatedData<CanvasDto>> => {
    const url = new URL(
      tenantApiUrl(tenantId, API_ENDPOINTS.CANVAS.GET_ALL),
      window.location.origin,
    );
    if (page !== undefined) {
      url.searchParams.set("page", page.toString());
    }
    if (pageSize !== undefined) {
      url.searchParams.set("page_size", pageSize.toString());
    }
    const response = await ApiService.get({
      url: url.toString(),
      onlyBody: true,
    });
    const paginatedResponse = response as PaginatedDataResponse<CanvasDto>;

    return {
      data: paginatedResponse.data,
      totalPages: paginatedResponse.total_pages,
      page: paginatedResponse.page,
      pageSize: paginatedResponse.page_size,
      totalItems: paginatedResponse.total_items,
    };
  },

  updateCanvas: async (
    tenantId: string,
    {
      canvasId,
      payload,
    }: {
      canvasId: string;
      payload: Partial<CanvasDto>;
    },
  ): Promise<CanvasDto> => {
    const response = await ApiService.put({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CANVAS.UPDATE.replace(":id", canvasId),
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CanvasDto;
  },

  createCanvas: async (
    tenantId: string,
    payload: NewCanvasDto,
  ): Promise<CanvasDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.CANVAS.CREATE),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CanvasDto;
  },

  deleteCanvas: async (tenantId: string, canvasId: string): Promise<void> => {
    await ApiService.delete({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CANVAS.DELETE.replace(":id", canvasId),
      ),
    });
  },
};

// Query hooks
export const useCanvas = (canvasId: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: canvasKeys.detail(tenantId, canvasId),
    queryFn: () => canvasApi.getCanvas(tenantId, canvasId),
    enabled: !!canvasId,
  });
};

export const useCanvasCapabilities = (): ResourceCapabilities => {
  const params = useParams();
  const canvasQuery = useCanvas(params.id!);
  if (!canvasQuery.data) {
    throw new Error("Canvas capabilities were read before the canvas loaded");
  }
  return canvasQuery.data.capabilities;
};

export const useAllCanvases = (page?: number, pageSize?: number) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: canvasKeys.listPaginated(tenantId, page ?? 1, pageSize ?? 10),
    queryFn: () => canvasApi.getAllCanvases(tenantId, page, pageSize),
    placeholderData: keepPreviousData,
  });
};

export const useUpdateCanvas = (callbacks?: {
  onSuccess?: (data: CanvasDto) => void;
  onError?: (error: Error) => void;
}) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (variables: { canvasId: string; payload: Partial<CanvasDto> }) =>
      canvasApi.updateCanvas(tenantId, variables),
    onSuccess: (data) => {
      queryClient.setQueryData(canvasKeys.detail(tenantId, data.id!), data);
      queryClient.invalidateQueries({
        queryKey: canvasKeys.lists(tenantId),
      });
      callbacks?.onSuccess?.(data);
    },
    onError: (error) => {
      console.error("Failed to update canvas:", error);
      callbacks?.onError?.(error);
    },
  });
};

export const useCreateCanvas = (callbacks?: {
  onSuccess?: (data: CanvasDto) => void;
  onError?: (error: Error) => void;
}) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (payload: NewCanvasDto) =>
      canvasApi.createCanvas(tenantId, payload),
    onSuccess: (data) => {
      // Add the new canvas to the cache
      queryClient.setQueryData(canvasKeys.detail(tenantId, data.id!), data);

      // Invalidate the canvases list to refetch
      queryClient.invalidateQueries({
        queryKey: canvasKeys.lists(tenantId),
      });

      // Call custom success callback if provided
      if (callbacks?.onSuccess) {
        callbacks.onSuccess(data);
      }
    },
    onError: (error) => {
      console.error("Failed to create canvas:", error);

      // Call custom error callback if provided
      if (callbacks?.onError) {
        callbacks.onError(error);
      }
    },
  });
};

export const useDeleteCanvas = (callbacks?: {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (canvasId: string) => canvasApi.deleteCanvas(tenantId, canvasId),
    onSuccess: (_, canvasId) => {
      // Remove the deleted canvas from the cache
      queryClient.removeQueries({
        queryKey: canvasKeys.detail(tenantId, canvasId),
      });

      // Invalidate the canvases list to refetch
      queryClient.invalidateQueries({
        queryKey: canvasKeys.lists(tenantId),
      });

      // Call custom success callback if provided
      if (callbacks?.onSuccess) {
        callbacks.onSuccess();
      }
    },
    onError: (error) => {
      console.error("Failed to delete canvas:", error);

      // Call custom error callback if provided
      if (callbacks?.onError) {
        callbacks.onError(error);
      }
    },
  });
};
