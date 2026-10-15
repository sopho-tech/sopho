import { useCallback } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import {
  CellDto,
  CreateCellDto,
  ExecuteCellResponseDto,
  CellExecutionResultDto,
  CellType,
} from "src/components/Notebook/Cell/dto";
import { notebookKeys } from "src/api/notebook/queries";
import { dashboardKeys } from "src/api/dashboard/queries";
import { canvasKeys } from "src/api/canvas/queries";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

type CellMovementType = "UP" | "DOWN" | "TOP" | "BOTTOM";

export const cellKeys = {
  all: (tenantId: string) => [tenantId, "cells"] as const,
  details: (tenantId: string) => [...cellKeys.all(tenantId), "detail"] as const,
  detail: (tenantId: string, id: string) =>
    [...cellKeys.details(tenantId), id] as const,
  execution: (tenantId: string, cellId: string) =>
    [...cellKeys.all(tenantId), "execution", cellId] as const,
};

const cellApi = {
  getCell: async (tenantId: string, cellId: string): Promise<CellDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CELL.GET_BY_ID.replace(":id", cellId),
      ),
      onlyBody: true,
    });
    return response as CellDto;
  },

  updateCell: async (
    tenantId: string,
    {
      cellId,
      payload,
    }: {
      cellId: string;
      payload: CellDto;
    },
  ): Promise<CellDto> => {
    const response = await ApiService.put({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CELL.UPDATE.replace(":id", cellId),
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CellDto;
  },

  createCell: async (
    tenantId: string,
    payload: CreateCellDto,
  ): Promise<CellDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.CELL.CREATE),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CellDto;
  },

  deleteCell: async (tenantId: string, cellId: string): Promise<void> => {
    await ApiService.delete({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CELL.DELETE.replace(":id", cellId),
      ),
    });
  },

  executeCell: async (
    tenantId: string,
    cellId: string,
  ): Promise<ExecuteCellResponseDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CELL.EXECUTE.replace(":id", cellId),
      ),
    });
    return response as ExecuteCellResponseDto;
  },

  executeCellPreview: async (
    tenantId: string,
    {
      cellId,
      content,
      cellType,
    }: {
      cellId: string;
      content: string;
      cellType: string;
    },
  ): Promise<ExecuteCellResponseDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CELL.EXECUTE_PREVIEW.replace(":id", cellId),
      ),
      data: { content, cell_type: cellType },
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ExecuteCellResponseDto;
  },

  reorderCell: async (
    tenantId: string,
    {
      cellId,
      movementType,
    }: {
      cellId: string;
      movementType: CellMovementType;
    },
  ): Promise<CellDto> => {
    const response = await ApiService.patch({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CELL.REORDER.replace(":id", cellId),
      ),
      data: { movement_type: movementType },
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CellDto;
  },
};

export const getCell = (tenantId: string, cellId: string) =>
  cellApi.getCell(tenantId, cellId);

export const useCell = (cellId: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: cellKeys.detail(tenantId, cellId),
    queryFn: () => cellApi.getCell(tenantId, cellId),
    enabled: !!cellId,
  });
};

export const useUpdateCell = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (variables: { cellId: string; payload: CellDto }) =>
      cellApi.updateCell(tenantId, variables),
    onSuccess: (data, { cellId }) => {
      queryClient.setQueryData(cellKeys.detail(tenantId, cellId), data);
      void queryClient.invalidateQueries({
        queryKey: cellKeys.detail(tenantId, cellId),
      });
      void queryClient.invalidateQueries({
        queryKey: canvasKeys.details(tenantId),
      });
    },
    onError: (error) => {
      console.error("Failed to update cell:", error);
    },
  });
};

export const useCreateCell = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (payload: CreateCellDto) =>
      cellApi.createCell(tenantId, payload),
    onSuccess: (data) => {
      queryClient.setQueryData(cellKeys.detail(tenantId, data.id!), data);
      if (data.notebook_id) {
        void queryClient.invalidateQueries({
          queryKey: notebookKeys.detail(tenantId, data.notebook_id),
        });
        void queryClient.invalidateQueries({
          queryKey: [...notebookKeys.cells(tenantId), data.notebook_id],
        });
      }
      void queryClient.invalidateQueries({
        queryKey: canvasKeys.details(tenantId),
      });
    },
    onError: (error) => {
      console.error("Failed to create cell:", error);
    },
  });
};

export const useDeleteCell = (notebookId?: string, canvasId?: string) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (cellId: string) => cellApi.deleteCell(tenantId, cellId),
    onSuccess: (_, cellId) => {
      queryClient.removeQueries({
        queryKey: cellKeys.detail(tenantId, cellId),
      });
      queryClient.removeQueries({
        queryKey: cellKeys.execution(tenantId, cellId),
      });

      if (notebookId) {
        void queryClient.invalidateQueries({
          queryKey: notebookKeys.detail(tenantId, notebookId),
        });
        void queryClient.invalidateQueries({
          queryKey: notebookKeys.cellsByNotebook(
            tenantId,
            notebookId,
            CellType.CHART,
          ),
        });
        void queryClient.invalidateQueries({
          queryKey: notebookKeys.cellsByNotebook(
            tenantId,
            notebookId,
            CellType.SQL,
          ),
        });
      }

      if (canvasId) {
        void queryClient.invalidateQueries({
          queryKey: dashboardKeys.byCanvas(tenantId, canvasId),
        });
      }
      void queryClient.invalidateQueries({
        queryKey: canvasKeys.details(tenantId),
      });
    },
    onError: (error) => {
      console.error("Failed to delete cell:", error);
    },
  });
};

export const useExecuteCell = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (cellId: string) => cellApi.executeCell(tenantId, cellId),
    onSuccess: (data, cellId) => {
      queryClient.setQueryData(cellKeys.execution(tenantId, cellId), {
        status: "success",
        data: data,
      } satisfies CellExecutionResultDto);
    },
    onError: (error, cellId) => {
      queryClient.setQueryData(cellKeys.execution(tenantId, cellId), {
        status: "error",
        error: error,
      } satisfies CellExecutionResultDto);
    },
  });
};

export const useExecuteCellPreview = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (variables: {
      cellId: string;
      content: string;
      cellType: string;
    }) => cellApi.executeCellPreview(tenantId, variables),
    onSuccess: (data, { cellId }) => {
      queryClient.setQueryData(cellKeys.execution(tenantId, cellId), {
        status: "success",
        data: data,
      } satisfies CellExecutionResultDto);
    },
    onError: (error, { cellId }) => {
      queryClient.setQueryData(cellKeys.execution(tenantId, cellId), {
        status: "error",
        error: error,
      } satisfies CellExecutionResultDto);
    },
  });
};

export const useCellExecutionResult = (cellId: string) => {
  const tenantId = useTenantId();
  return useQuery<CellExecutionResultDto>({
    queryKey: cellKeys.execution(tenantId, cellId),
    queryFn: () =>
      Promise.reject(new Error("Execution results are cache-only")),
    enabled: false,
  });
};

export const useClearCellOutput = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();
  return useCallback(
    (cellId: string) => {
      queryClient.setQueryData(cellKeys.execution(tenantId, cellId), null);
    },
    [queryClient, tenantId],
  );
};

export const useReorderCell = (canvasId?: string) => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (variables: {
      cellId: string;
      movementType: CellMovementType;
    }) => cellApi.reorderCell(tenantId, variables),
    onSuccess: (data) => {
      queryClient.setQueryData(cellKeys.detail(tenantId, data.id!), data);
      if (data.notebook_id) {
        void queryClient.invalidateQueries({
          queryKey: notebookKeys.detail(tenantId, data.notebook_id),
        });
        void queryClient.invalidateQueries({
          queryKey: [...notebookKeys.cells(tenantId), data.notebook_id],
        });
      }
      if (canvasId) {
        void queryClient.invalidateQueries({
          queryKey: dashboardKeys.byCanvas(tenantId, canvasId),
        });
      }
      void queryClient.invalidateQueries({
        queryKey: canvasKeys.details(tenantId),
      });
    },
    onError: (error) => {
      console.error("Failed to reorder cell:", error);
    },
  });
};
