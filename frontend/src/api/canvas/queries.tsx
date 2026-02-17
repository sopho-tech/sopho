import {
  useQuery,
  useMutation,
  useQueryClient,
  keepPreviousData,
} from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { CanvasDto } from "src/components/Canvases/dto";
import { PaginatedData, PaginatedDataResponse } from "../dto";

// Query keys for canvases
export const canvasKeys = {
  all: ["canvases"] as const,
  lists: () => [...canvasKeys.all, "list"] as const,
  list: (filters: string) => [...canvasKeys.lists(), { filters }] as const,
  listPaginated: (page: number, pageSize: number) =>
    [...canvasKeys.lists(), { page, pageSize }] as const,
  details: () => [...canvasKeys.all, "detail"] as const,
  detail: (id: string) => [...canvasKeys.details(), id] as const,
};

// API functions
const canvasApi = {
  getCanvas: async (canvasId: string): Promise<CanvasDto> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.CANVAS.GET_BY_ID?.replace(":id", canvasId) || `/canvas/${canvasId}`,
      onlyBody: true,
    });
    return response as CanvasDto;
  },

  getAllCanvases: async (
    page?: number,
    pageSize?: number
  ): Promise<PaginatedData<CanvasDto>> => {
    const url = new URL(API_ENDPOINTS.CANVAS.GET_ALL, window.location.origin);
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

  updateCanvas: async ({
    canvasId,
    payload,
  }: {
    canvasId: string;
    payload: Partial<CanvasDto>;
  }): Promise<CanvasDto> => {
    const response = await ApiService.put({
      url: API_ENDPOINTS.CANVAS.UPDATE.replace(":id", canvasId),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CanvasDto;
  },

  createCanvas: async (payload: Omit<CanvasDto, "id">): Promise<CanvasDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.CANVAS.CREATE,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CanvasDto;
  },

  deleteCanvas: async (canvasId: string): Promise<void> => {
    const response = await fetch(
      API_ENDPOINTS.CANVAS.DELETE?.replace(":id", canvasId) || `/canvas/${canvasId}`,
      {
        method: "DELETE",
        credentials: "include",
      }
    );

    if (!response.ok) {
      throw new Error(`Failed to delete canvas: ${response.statusText}`);
    }
  },
};

// Query hooks
export const useCanvas = (canvasId: string) => {
  return useQuery({
    queryKey: canvasKeys.detail(canvasId),
    queryFn: () => canvasApi.getCanvas(canvasId),
    enabled: !!canvasId,
  });
};

export const useAllCanvases = (page?: number, pageSize?: number) => {
  return useQuery({
    queryKey: canvasKeys.listPaginated(page ?? 1, pageSize ?? 10),
    queryFn: () => canvasApi.getAllCanvases(page, pageSize),
    placeholderData: keepPreviousData,
  });
};

export const useUpdateCanvas = (callbacks?: {
  onSuccess?: (data: CanvasDto) => void;
  onError?: (error: Error) => void;
}) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: canvasApi.updateCanvas,
    onSuccess: (data) => {
      queryClient.setQueryData(canvasKeys.detail(data.id!), data);
      queryClient.invalidateQueries({
        queryKey: canvasKeys.lists(),
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

  return useMutation({
    mutationFn: canvasApi.createCanvas,
    onSuccess: (data) => {
      // Add the new canvas to the cache
      queryClient.setQueryData(canvasKeys.detail(data.id!), data);

      // Invalidate the canvases list to refetch
      queryClient.invalidateQueries({
        queryKey: canvasKeys.lists(),
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

  return useMutation({
    mutationFn: canvasApi.deleteCanvas,
    onSuccess: (_, canvasId) => {
      // Remove the deleted canvas from the cache
      queryClient.removeQueries({
        queryKey: canvasKeys.detail(canvasId),
      });

      // Invalidate the canvases list to refetch
      queryClient.invalidateQueries({
        queryKey: canvasKeys.lists(),
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
