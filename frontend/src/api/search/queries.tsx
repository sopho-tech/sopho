import { useQuery } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { EntityType } from "./constants";

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
  all: ["search"] as const,
  search: (query: string, filters: EntityType[]) =>
    [...searchKeys.all, { query, filters }] as const,
};

const searchApi = {
  search: async (
    query: string,
    filters: EntityType[]
  ): Promise<SearchResultItemDto[]> => {
    const url = new URL(API_ENDPOINTS.SEARCH.SEARCH, window.location.origin);
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
  return useQuery({
    queryKey: searchKeys.search(query, filters),
    queryFn: () => searchApi.search(query, filters),
    enabled: filters.length > 0,
  });
};
