import { useQuery } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { EntityType } from "./constants";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

export type SearchRequestDto = {
  query?: string;
  filters?: string[];
};

export type SearchResultItemDto = {
  id: string;
  name: string | null;
  entity_type: EntityType;
  updated_at: string;
  canvas_id: string | null;
};

export type SearchResponseDto = {
  data: SearchResultItemDto[];
};

export const searchKeys = {
  all: (tenantId: string) => [tenantId, "search"] as const,
  search: (tenantId: string, query: string, filters: EntityType[]) =>
    [...searchKeys.all(tenantId), { query, filters }] as const,
};

const searchApi = {
  search: async (
    tenantId: string,
    query: string,
    filters: EntityType[]
  ): Promise<SearchResultItemDto[]> => {
    const url = new URL(
      tenantApiUrl(tenantId, API_ENDPOINTS.SEARCH.SEARCH),
      window.location.origin,
    );
    if (query) {
      url.searchParams.set("query", query);
    }
    if (filters.length > 0) {
      url.searchParams.set("filters", filters.join(","));
    }

    const response = await ApiService.get<SearchResponseDto>({
      url: url.toString(),
      onlyBody: true,
    });
    const searchResponse = response as SearchResponseDto;

    return searchResponse.data;
  },
};

export const useSearch = (query: string, filters: EntityType[]) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: searchKeys.search(tenantId, query, filters),
    queryFn: () => searchApi.search(tenantId, query, filters),
    enabled: filters.length > 0,
  });
};
