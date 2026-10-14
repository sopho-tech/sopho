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
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

// Query keys for notebooks
export const notebookKeys = {
  all: (tenantId: string) => [tenantId, "notebooks"] as const,
  lists: (tenantId: string) =>
    [...notebookKeys.all(tenantId), "list"] as const,
  listPaginated: (tenantId: string, page: number, pageSize: number) =>
    [...notebookKeys.lists(tenantId), { page, pageSize }] as const,
  details: (tenantId: string) =>
    [...notebookKeys.all(tenantId), "detail"] as const,
  detail: (tenantId: string, id: string) =>
    [...notebookKeys.details(tenantId), id] as const,
  byCanvas: (tenantId: string, canvasId: string) =>
    [...notebookKeys.all(tenantId), "byCanvas", canvasId] as const,
  cells: (tenantId: string) =>
    [...notebookKeys.all(tenantId), "cells"] as const,
  cellsByNotebook: (tenantId: string, notebookId: string, cellType?: string) =>
    [...notebookKeys.cells(tenantId), notebookId, { cellType }] as const,
};

// API functions
const notebookApi = {
  getNotebook: async (
    tenantId: string,
    notebookId: string,
  ): Promise<NotebookDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.NOTEBOOK.GET_BY_ID.replace(":id", notebookId),
      ),
      onlyBody: true,
    });
    return response as NotebookDto;
  },

  getAllNotebooks: async (
    tenantId: string,
    page?: number,
    pageSize?: number
  ): Promise<PaginatedData<NotebookDto>> => {
    const url = new URL(
      tenantApiUrl(tenantId, API_ENDPOINTS.NOTEBOOK.GET_ALL),
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
    tenantId: string,
    payload: Omit<NotebookDto, "id">
  ): Promise<NotebookDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.NOTEBOOK.CREATE),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as NotebookDto;
  },

  getNotebooksByCanvasId: async (
    tenantId: string,
    canvasId: string,
  ): Promise<NotebookDto[]> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.NOTEBOOK.GET_BY_CANVAS_ID.replace(":canvas_id", canvasId),
      ),
      onlyBody: true,
    });
    return response as NotebookDto[];
  },

  getCellsByNotebookId: async (
    tenantId: string,
    notebookId: string,
    cellType?: string
  ): Promise<NotebookDto> => {
    const url = new URL(
      tenantApiUrl(
        tenantId,
        API_ENDPOINTS.NOTEBOOK.GET_CELLS_BY_NOTEBOOK_ID.replace(":id", notebookId),
      ),
      window.location.origin
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
  const tenantId = useTenantId();
  return useQuery({
    queryKey: notebookKeys.detail(tenantId, notebookId),
    queryFn: () => notebookApi.getNotebook(tenantId, notebookId),
    enabled: !!notebookId,
  });
};

export const useAllNotebooks = (page?: number, pageSize?: number) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: notebookKeys.listPaginated(tenantId, page ?? 1, pageSize ?? 10),
    queryFn: () => notebookApi.getAllNotebooks(tenantId, page, pageSize),
    placeholderData: keepPreviousData,
  });
};

export const useNotebooksByCanvasId = (canvasId: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: notebookKeys.byCanvas(tenantId, canvasId),
    queryFn: () => notebookApi.getNotebooksByCanvasId(tenantId, canvasId),
    enabled: !!canvasId,
  });
};

export const useCellsByNotebookId = (notebookId: string, cellType?: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: notebookKeys.cellsByNotebook(tenantId, notebookId, cellType),
    queryFn: () =>
      notebookApi.getCellsByNotebookId(tenantId, notebookId, cellType),
    enabled: !!notebookId,
  });
};

// Mutation hooks
export const useCreateNotebook = (callbacks?: {
  onSuccess?: (data: NotebookDto) => void;
  onError?: (error: Error) => void;
}) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (payload: Omit<NotebookDto, "id">) =>
      notebookApi.createNotebook(tenantId, payload),
    onSuccess: (data) => {
      // Add the new notebook to the cache
      queryClient.setQueryData(notebookKeys.detail(tenantId, data.id!), data);

      // Invalidate the notebooks list to refetch
      queryClient.invalidateQueries({
        queryKey: notebookKeys.lists(tenantId),
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
