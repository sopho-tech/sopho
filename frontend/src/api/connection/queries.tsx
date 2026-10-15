import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { CreatedByFilter } from "src/api/dto";
import { ConnectionDto } from "src/components/Connection/dto";
import { type ExecuteCellResponseDto } from "src/components/Notebook/Cell/dto";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

export type CreateConnectionDto = Omit<
  ConnectionDto,
  | "id"
  | "created_at"
  | "updated_at"
  | "capabilities"
  | "created_by"
  | "modified_by"
> & { password: string };

export type UpdateConnectionDto = Pick<
  ConnectionDto,
  "name" | "description" | "host" | "port" | "database" | "schema" | "username"
> & { password: string | null };

export type QueryExecutionResultDto =
  | { status: "success"; data: ExecuteCellResponseDto }
  | { status: "error"; error: unknown };

export const connectionKeys = {
  all: (tenantId: string) => [tenantId, "connections"] as const,
  lists: (tenantId: string) =>
    [...connectionKeys.all(tenantId), "list"] as const,
  list: (tenantId: string, createdBy: CreatedByFilter | null) =>
    [...connectionKeys.lists(tenantId), { createdBy }] as const,
  details: (tenantId: string) =>
    [...connectionKeys.all(tenantId), "detail"] as const,
  detail: (tenantId: string, id: string) =>
    [...connectionKeys.details(tenantId), id] as const,
  queryExecution: (tenantId: string, connectionId: string, query: string) =>
    [
      ...connectionKeys.all(tenantId),
      "query-execution",
      connectionId,
      query,
    ] as const,
};

const connectionApi = {
  getConnection: async (
    tenantId: string,
    connectionId: string,
  ): Promise<ConnectionDto> => {
    const response = await ApiService.get({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONNECTION.GET_BY_ID.replace(":id", connectionId),
      ),
      onlyBody: true,
    });
    return response as ConnectionDto;
  },

  getAllConnections: async (
    tenantId: string,
    createdBy: CreatedByFilter | null,
  ): Promise<ConnectionDto[]> => {
    const url = new URL(
      tenantApiUrl(tenantId, API_ENDPOINTS.CONNECTION.GET_ALL),
      window.location.origin,
    );
    if (createdBy !== null) {
      url.searchParams.set("created_by", createdBy);
    }
    const response = await ApiService.get({
      url: url.toString(),
      onlyBody: true,
    });
    return response as ConnectionDto[];
  },

  updateConnection: async (
    tenantId: string,
    {
      connectionId,
      payload,
    }: {
      connectionId: string;
      payload: UpdateConnectionDto;
    },
  ): Promise<ConnectionDto> => {
    const response = await ApiService.put({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONNECTION.UPDATE.replace(":id", connectionId),
      ),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConnectionDto;
  },

  createConnection: async (
    tenantId: string,
    payload: CreateConnectionDto,
  ): Promise<ConnectionDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(tenantId, API_ENDPOINTS.CONNECTION.CREATE),
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
    return response as ConnectionDto;
  },

  executeQuery: async (
    tenantId: string,
    {
      connectionId,
      query,
    }: {
      connectionId: string;
      query: string;
    },
  ): Promise<ExecuteCellResponseDto> => {
    const response = await ApiService.post({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONNECTION.EXECUTE_QUERY.replace(":id", connectionId),
      ),
      data: { query },
      headers: { "Content-Type": "application/json" },
    });
    return response as ExecuteCellResponseDto;
  },

  deleteConnection: async (
    tenantId: string,
    connectionId: string,
  ): Promise<void> => {
    await ApiService.delete({
      url: tenantApiUrl(
        tenantId,
        API_ENDPOINTS.CONNECTION.DELETE.replace(":id", connectionId),
      ),
    });
  },
};

export const useConnection = (connectionId: string) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: connectionKeys.detail(tenantId, connectionId),
    queryFn: () => connectionApi.getConnection(tenantId, connectionId),
    enabled: !!connectionId,
  });
};

export const useConnections = (createdBy: CreatedByFilter | null) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: connectionKeys.list(tenantId, createdBy),
    queryFn: () => connectionApi.getAllConnections(tenantId, createdBy),
  });
};

export const useUpdateConnection = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (variables: {
      connectionId: string;
      payload: UpdateConnectionDto;
    }) => connectionApi.updateConnection(tenantId, variables),
    onSuccess: (data, { connectionId }) => {
      queryClient.setQueryData(
        connectionKeys.detail(tenantId, connectionId),
        data,
      );

      void queryClient.invalidateQueries({
        queryKey: connectionKeys.lists(tenantId),
      });
    },
    onError: (error) => {
      console.error("Failed to update connection:", error);
    },
  });
};

export const useCreateConnection = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (payload: CreateConnectionDto) =>
      connectionApi.createConnection(tenantId, payload),
    onSuccess: (data) => {
      queryClient.setQueryData(connectionKeys.detail(tenantId, data.id), data);

      void queryClient.invalidateQueries({
        queryKey: connectionKeys.lists(tenantId),
      });
    },
    onError: (error) => {
      console.error("Failed to create connection:", error);
    },
  });
};

export const useExecuteQuery = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (variables: { connectionId: string; query: string }) =>
      connectionApi.executeQuery(tenantId, variables),
    onSuccess: (data, { connectionId, query }) => {
      queryClient.setQueryData(
        connectionKeys.queryExecution(tenantId, connectionId, query),
        { status: "success", data } satisfies QueryExecutionResultDto,
      );
    },
    onError: (error, { connectionId, query }) => {
      queryClient.setQueryData(
        connectionKeys.queryExecution(tenantId, connectionId, query),
        { status: "error", error } satisfies QueryExecutionResultDto,
      );
    },
  });
};

export const useQueryExecutionResult = (
  connectionId: string,
  query: string,
) => {
  const tenantId = useTenantId();
  return useQuery<QueryExecutionResultDto>({
    queryKey: connectionKeys.queryExecution(tenantId, connectionId, query),
    queryFn: () =>
      Promise.reject(new Error("Execution results are cache-only")),
    enabled: false,
  });
};

export const useDeleteConnection = () => {
  const queryClient = useQueryClient();
  const tenantId = useTenantId();

  return useMutation({
    mutationFn: (connectionId: string) =>
      connectionApi.deleteConnection(tenantId, connectionId),
    onSuccess: (_, connectionId) => {
      queryClient.removeQueries({
        queryKey: connectionKeys.detail(tenantId, connectionId),
      });

      void queryClient.invalidateQueries({
        queryKey: connectionKeys.lists(tenantId),
      });
    },
    onError: (error) => {
      console.error("Failed to delete connection:", error);
    },
  });
};
