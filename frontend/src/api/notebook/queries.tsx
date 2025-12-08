import {
  useQuery,
  useMutation,
  useQueryClient,
  keepPreviousData,
} from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { NotebookDto } from "src/components/Notebook/dto";
import { PaginatedData, PaginatedDataResponse } from "../dto";

// Query keys for notebooks
export const notebookKeys = {
  all: ["notebooks"] as const,
  lists: () => [...notebookKeys.all, "list"] as const,
  list: (filters: string) => [...notebookKeys.lists(), { filters }] as const,
  listPaginated: (page: number, pageSize: number) =>
    [...notebookKeys.lists(), { page, pageSize }] as const,
  details: () => [...notebookKeys.all, "detail"] as const,
  detail: (id: string) => [...notebookKeys.details(), id] as const,
  byCanvas: (canvasId: string) =>
    [...notebookKeys.all, "byCanvas", canvasId] as const,
  cells: () => [...notebookKeys.all, "cells"] as const,
  cellsByNotebook: (notebookId: string, cellType?: string) =>
    [...notebookKeys.cells(), notebookId, { cellType }] as const,
};

// API functions
const notebookApi = {
  getNotebook: async (notebookId: string): Promise<NotebookDto> => {
    const response = await ApiService.get({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.NOTEBOOK.GET_BY_ID?.replace(":id", notebookId) || `/notebooks/${notebookId}`}`,
      onlyBody: true,
    });
    return response as NotebookDto;
  },

  getAllNotebooks: async (
    page?: number,
    pageSize?: number
  ): Promise<PaginatedData<NotebookDto>> => {
    const url = new URL(
      `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.NOTEBOOK.GET_ALL}`
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
    const paginatedResponse = response as PaginatedDataResponse<NotebookDto>;

    return {
      data: paginatedResponse.data,
      totalPages: paginatedResponse.total_pages,
      page: paginatedResponse.page,
      pageSize: paginatedResponse.page_size,
      totalItems: paginatedResponse.total_items,
    };
  },

  createNotebook: async (
    payload: Omit<NotebookDto, "id">
  ): Promise<NotebookDto> => {
    const response = await ApiService.post({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.NOTEBOOK.CREATE}`,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as NotebookDto;
  },

  updateNotebook: async ({
    notebookId,
    payload,
  }: {
    notebookId: string;
    payload: Partial<NotebookDto>;
  }): Promise<NotebookDto> => {
    const response = await ApiService.put({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.NOTEBOOK.UPDATE.replace(":id", notebookId)}`,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as NotebookDto;
  },

  deleteNotebook: async (notebookId: string): Promise<void> => {
    const response = await fetch(
      `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.NOTEBOOK.DELETE?.replace(":id", notebookId) || `/notebooks/${notebookId}`}`,
      {
        method: "DELETE",
        credentials: "include",
      }
    );

    if (!response.ok) {
      throw new Error(`Failed to delete notebook: ${response.statusText}`);
    }
  },

  getNotebooksByCanvasId: async (canvasId: string): Promise<NotebookDto[]> => {
    const response = await ApiService.get({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.NOTEBOOK.GET_BY_CANVAS_ID?.replace(":canvas_id", canvasId) || `/notebooks/canvas/${canvasId}`}`,
      onlyBody: true,
    });
    return response as NotebookDto[];
  },

  getCellsByNotebookId: async (
    notebookId: string,
    cellType?: string
  ): Promise<NotebookDto> => {
    const url = new URL(
      `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.NOTEBOOK.GET_CELLS_BY_NOTEBOOK_ID?.replace(":id", notebookId) || `/notebooks/${notebookId}/cells`}`
    );
    if (cellType) {
      url.searchParams.set("cell_type", cellType);
    }
    const response = await ApiService.get({
      url: url.toString(),
      onlyBody: true,
    });
    return response as NotebookDto;
  },
};

// Query hooks
export const useNotebook = (notebookId: string) => {
  return useQuery({
    queryKey: notebookKeys.detail(notebookId),
    queryFn: () => notebookApi.getNotebook(notebookId),
    enabled: !!notebookId,
  });
};

export const useAllNotebooks = (page?: number, pageSize?: number) => {
  return useQuery({
    queryKey: notebookKeys.listPaginated(page ?? 1, pageSize ?? 10),
    queryFn: () => notebookApi.getAllNotebooks(page, pageSize),
    placeholderData: keepPreviousData,
  });
};

export const useNotebooksByCanvasId = (canvasId: string) => {
  return useQuery({
    queryKey: notebookKeys.byCanvas(canvasId),
    queryFn: () => notebookApi.getNotebooksByCanvasId(canvasId),
    enabled: !!canvasId,
  });
};

export const useCellsByNotebookId = (notebookId: string, cellType?: string) => {
  return useQuery({
    queryKey: notebookKeys.cellsByNotebook(notebookId, cellType),
    queryFn: () => notebookApi.getCellsByNotebookId(notebookId, cellType),
    enabled: !!notebookId,
  });
};

// Mutation hooks
export const useCreateNotebook = (callbacks?: {
  onSuccess?: (data: NotebookDto) => void;
  onError?: (error: Error) => void;
}) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: notebookApi.createNotebook,
    onSuccess: (data) => {
      // Add the new notebook to the cache
      queryClient.setQueryData(notebookKeys.detail(data.id!), data);

      // Invalidate the notebooks list to refetch
      queryClient.invalidateQueries({
        queryKey: notebookKeys.lists(),
      });

      // Call custom success callback if provided
      if (callbacks?.onSuccess) {
        callbacks.onSuccess(data);
      }
    },
    onError: (error) => {
      console.error("Failed to create notebook:", error);

      // Call custom error callback if provided
      if (callbacks?.onError) {
        callbacks.onError(error);
      }
    },
  });
};

export const useUpdateNotebook = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: notebookApi.updateNotebook,
    onSuccess: (data, { notebookId }) => {
      // Update the specific notebook query
      queryClient.setQueryData(notebookKeys.detail(notebookId), data);

      // Invalidate the notebooks list to refetch
      queryClient.invalidateQueries({
        queryKey: notebookKeys.lists(),
      });
    },
    onError: (error) => {
      console.error("Failed to update notebook:", error);
    },
  });
};

export const useDeleteNotebook = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: notebookApi.deleteNotebook,
    onSuccess: (_, notebookId) => {
      // Remove the notebook from cache
      queryClient.removeQueries({
        queryKey: notebookKeys.detail(notebookId),
      });

      // Invalidate the notebooks list to refetch
      queryClient.invalidateQueries({
        queryKey: notebookKeys.lists(),
      });
    },
    onError: (error) => {
      console.error("Failed to delete notebook:", error);
    },
  });
};
