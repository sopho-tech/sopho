import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { ConnectionDto } from "src/components/Connection/dto";

export type CreateConnectionDto = Omit<
  ConnectionDto,
  "id" | "created_at" | "updated_at"
>;

export const connectionKeys = {
  all: ["connections"] as const,
  lists: () => [...connectionKeys.all, "list"] as const,
  list: (filters: string) => [...connectionKeys.lists(), { filters }] as const,
  details: () => [...connectionKeys.all, "detail"] as const,
  detail: (id: string) => [...connectionKeys.details(), id] as const,
};

const connectionApi = {
  getConnection: async (connectionId: string): Promise<ConnectionDto> => {
    const response = await ApiService.get({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CONNECTION.GET_BY_ID?.replace(":id", connectionId) || `/api/v1/connection/${connectionId}`}`,
      onlyBody: true,
    });
    return response as ConnectionDto;
  },

  getAllConnections: async (): Promise<ConnectionDto[]> => {
    const response = await ApiService.get({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CONNECTION.GET_ALL}`,
      onlyBody: true,
    });
    return response as ConnectionDto[];
  },

  updateConnection: async ({
    connectionId,
    payload,
  }: {
    connectionId: string;
    payload: Partial<ConnectionDto>;
  }): Promise<ConnectionDto> => {
    const response = await ApiService.put({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CONNECTION.UPDATE.replace(":id", connectionId)}`,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConnectionDto;
  },

  createConnection: async (
    payload: CreateConnectionDto,
  ): Promise<ConnectionDto> => {
    const response = await ApiService.post({
      url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CONNECTION.CREATE}`,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConnectionDto;
  },

  deleteConnection: async (connectionId: string): Promise<void> => {
    const response = await fetch(
      `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.CONNECTION.DELETE?.replace(":id", connectionId) || `/api/v1/connection/${connectionId}`}`,
      {
        method: "DELETE",
        credentials: "include",
      },
    );

    if (!response.ok) {
      throw new Error(`Failed to delete connection: ${response.statusText}`);
    }
  },
};

export const useConnection = (connectionId: string) => {
  return useQuery({
    queryKey: connectionKeys.detail(connectionId),
    queryFn: () => connectionApi.getConnection(connectionId),
    enabled: !!connectionId,
  });
};

export const useConnections = () => {
  return useQuery({
    queryKey: connectionKeys.lists(),
    queryFn: () => connectionApi.getAllConnections(),
  });
};

export const useUpdateConnection = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: connectionApi.updateConnection,
    onSuccess: (data, { connectionId }) => {
      queryClient.setQueryData(connectionKeys.detail(connectionId), data);

      queryClient.invalidateQueries({
        queryKey: connectionKeys.lists(),
      });
    },
    onError: (error) => {
      console.error("Failed to update connection:", error);
    },
  });
};

export const useCreateConnection = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: connectionApi.createConnection,
    onSuccess: (data) => {
      queryClient.setQueryData(connectionKeys.detail(data.id), data);

      queryClient.invalidateQueries({
        queryKey: connectionKeys.lists(),
      });
    },
    onError: (error) => {
      console.error("Failed to create connection:", error);
    },
  });
};

export const useDeleteConnection = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: connectionApi.deleteConnection,
    onSuccess: (_, connectionId) => {
      queryClient.removeQueries({
        queryKey: connectionKeys.detail(connectionId),
      });

      queryClient.invalidateQueries({
        queryKey: connectionKeys.lists(),
      });
    },
    onError: (error) => {
      console.error("Failed to delete connection:", error);
    },
  });
};
