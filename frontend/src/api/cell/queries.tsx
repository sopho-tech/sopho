import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import {
  CellDto,
  CreateCellDto,
  ExecuteCellResponseDto,
} from "src/components/Notebooks/Notebook/Cell/dto";
import { notebookKeys } from "src/api/notebook/queries";

export const cellKeys = {
  all: ["cells"] as const,
  lists: () => [...cellKeys.all, "list"] as const,
  list: (filters: string) => [...cellKeys.lists(), { filters }] as const,
  details: () => [...cellKeys.all, "detail"] as const,
  detail: (id: string) => [...cellKeys.details(), id] as const,
};

const cellApi = {
  getCell: async (cellId: string): Promise<CellDto> => {
    const response = await ApiService.get({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CELL.GET_BY_ID?.replace(":id", cellId) || `/cells/${cellId}`}`,
      onlyBody: true,
    });
    return response as CellDto;
  },

  updateCell: async ({
    cellId,
    payload,
  }: {
    cellId: string;
    payload: CellDto;
  }): Promise<CellDto> => {
    const response = await ApiService.put({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CELL.UPDATE.replace(":id", cellId)}`,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CellDto;
  },

  createCell: async (payload: CreateCellDto): Promise<CellDto> => {
    const response = await ApiService.post({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CELL.CREATE}`,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as CellDto;
  },

  deleteCell: async (cellId: string): Promise<void> => {
    const response = await fetch(
      `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CELL.DELETE?.replace(":id", cellId) || `/cells/${cellId}`}`,
      {
        method: "DELETE",
        credentials: "include",
      }
    );

    if (!response.ok) {
      throw new Error(`Failed to delete cell: ${response.statusText}`);
    }
  },

  executeCell: async (cellId: string): Promise<ExecuteCellResponseDto> => {
    const response = await ApiService.post({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CELL.EXECUTE.replace(":id", cellId)}`,
    });
    return response as ExecuteCellResponseDto;
  },
};

export const useCell = (cellId: string) => {
  return useQuery({
    queryKey: cellKeys.detail(cellId),
    queryFn: () => cellApi.getCell(cellId),
    enabled: !!cellId,
  });
};

export const useUpdateCell = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: cellApi.updateCell,
    onSuccess: (data, { cellId }) => {
      queryClient.setQueryData(cellKeys.detail(cellId), data);
      queryClient.invalidateQueries({
        queryKey: cellKeys.detail(cellId),
      });
    },
    onError: (error) => {
      console.error("Failed to update cell:", error);
    },
  });
};

export const useCreateCell = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: cellApi.createCell,
    onSuccess: (data) => {
      queryClient.setQueryData(cellKeys.detail(data.id!), data);
      if (data.notebook_id) {
        queryClient.invalidateQueries({
          queryKey: notebookKeys.detail(data.notebook_id),
        });
      }
    },
    onError: (error) => {
      console.error("Failed to create cell:", error);
    },
  });
};

export const useDeleteCell = (notebookId?: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: cellApi.deleteCell,
    onSuccess: (_, cellId) => {
      queryClient.removeQueries({
        queryKey: cellKeys.detail(cellId),
      });

      if (notebookId) {
        queryClient.invalidateQueries({
          queryKey: notebookKeys.detail(notebookId),
        });
      }
    },
    onError: (error) => {
      console.error("Failed to delete cell:", error);
    },
  });
};

export const useExecuteCell = () => {
  return useMutation({
    mutationFn: cellApi.executeCell,
    onSuccess: (data) => {},
    onError: (error) => {
      console.error("Failed to execute cell:", error);
    },
  });
};
