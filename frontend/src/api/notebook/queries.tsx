import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { NotebookDto } from "src/components/Notebooks/dto";

// Query keys for notebooks
export const notebookKeys = {
  all: ["notebooks"] as const,
  lists: () => [...notebookKeys.all, "list"] as const,
  list: (filters: string) => [...notebookKeys.lists(), { filters }] as const,
  details: () => [...notebookKeys.all, "detail"] as const,
  detail: (id: string) => [...notebookKeys.details(), id] as const,
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

  getAllNotebooks: async (): Promise<NotebookDto[]> => {
    const response = await ApiService.get({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.NOTEBOOK.GET_ALL}`,
      onlyBody: true,
    });
    return response as NotebookDto[];
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
};

// Query hooks
export const useNotebook = (notebookId: string) => {
  return useQuery({
    queryKey: notebookKeys.detail(notebookId),
    queryFn: () => notebookApi.getNotebook(notebookId),
    enabled: !!notebookId,
  });
};

export const useAllNotebooks = () => {
  return useQuery({
    queryKey: notebookKeys.lists(),
    queryFn: notebookApi.getAllNotebooks,
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
