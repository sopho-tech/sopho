import { useCallback, useMemo } from "react";
import { useSearchParams } from "react-router";
import {
  UsageListParams,
  UsageRangeParams,
  UsageSortColumn,
  UsageSortDirection,
} from "src/api/ai_usage";
import {
  ALL_KEYS_OPTION,
  CONVERSATION_PAGE_SEARCH_PARAMETER,
  DEFAULT_RANGE_OPTION,
  DEFAULT_SORT_COLUMN,
  DEFAULT_SORT_DIRECTION,
  KEY_SEARCH_PARAMETER,
  OPPOSITE_SORT_DIRECTIONS,
  RANGE_SEARCH_PARAMETER,
  SEARCH_TEXT_SEARCH_PARAMETER,
  SORT_COLUMN_SEARCH_PARAMETER,
  SORT_DIRECTION_SEARCH_PARAMETER,
  USAGE_LIST_SEARCH_PARAMETERS,
} from "src/components/Settings/AIUsage/constants";
import { FIRST_PAGE } from "src/constants/pagination";

export function useUsageRange(): UsageRangeParams {
  const [searchParams] = useSearchParams();
  const lastDayCount =
    searchParams.get(RANGE_SEARCH_PARAMETER) ?? DEFAULT_RANGE_OPTION;
  const keySource = searchParams.get(KEY_SEARCH_PARAMETER);
  return useMemo(
    () => ({ lastDayCount, keySource }),
    [lastDayCount, keySource],
  );
}

export function useChangeUsageRange(): (rangeOption: string) => void {
  const [, setSearchParams] = useSearchParams();
  return useCallback(
    (rangeOption: string) =>
      setSearchParams((currentSearchParams) => {
        const nextSearchParams = new URLSearchParams(currentSearchParams);
        nextSearchParams.set(RANGE_SEARCH_PARAMETER, rangeOption);
        nextSearchParams.delete(CONVERSATION_PAGE_SEARCH_PARAMETER);
        return nextSearchParams;
      }),
    [setSearchParams],
  );
}

export function useChangeUsageKey(): (keyOption: string) => void {
  const [, setSearchParams] = useSearchParams();
  return useCallback(
    (keyOption: string) =>
      setSearchParams((currentSearchParams) => {
        const nextSearchParams = new URLSearchParams(currentSearchParams);
        if (keyOption === ALL_KEYS_OPTION) {
          nextSearchParams.delete(KEY_SEARCH_PARAMETER);
        } else {
          nextSearchParams.set(KEY_SEARCH_PARAMETER, keyOption);
        }
        nextSearchParams.delete(CONVERSATION_PAGE_SEARCH_PARAMETER);
        return nextSearchParams;
      }),
    [setSearchParams],
  );
}

export function searchWithoutUsageList(search: string): string {
  const searchParamsWithoutUsageList = new URLSearchParams(search);
  USAGE_LIST_SEARCH_PARAMETERS.forEach((usageListSearchParameter) =>
    searchParamsWithoutUsageList.delete(usageListSearchParameter),
  );
  return searchParamsWithoutUsageList.toString();
}

export function useUsageList(pageSize: number): {
  usageList: UsageListParams;
  changePage: (page: number) => void;
  changeSort: (sortColumn: UsageSortColumn) => void;
  changeSearch: (search: string) => void;
} {
  const [searchParams, setSearchParams] = useSearchParams();
  const requestedPage = searchParams.get(CONVERSATION_PAGE_SEARCH_PARAMETER);
  const page = requestedPage === null ? FIRST_PAGE : Number(requestedPage);
  const sortColumn = (searchParams.get(SORT_COLUMN_SEARCH_PARAMETER) ??
    DEFAULT_SORT_COLUMN) as UsageSortColumn;
  const sortDirection = (searchParams.get(SORT_DIRECTION_SEARCH_PARAMETER) ??
    DEFAULT_SORT_DIRECTION) as UsageSortDirection;
  const search = searchParams.get(SEARCH_TEXT_SEARCH_PARAMETER) ?? "";
  const usageList = useMemo(
    () => ({ page, pageSize, search, sortColumn, sortDirection }),
    [page, pageSize, search, sortColumn, sortDirection],
  );

  const changePage = useCallback(
    (nextPage: number) =>
      setSearchParams((currentSearchParams) => {
        const nextSearchParams = new URLSearchParams(currentSearchParams);
        nextSearchParams.set(
          CONVERSATION_PAGE_SEARCH_PARAMETER,
          String(nextPage),
        );
        return nextSearchParams;
      }),
    [setSearchParams],
  );
  const changeSort = useCallback(
    (nextSortColumn: UsageSortColumn) =>
      setSearchParams((currentSearchParams) => {
        const nextSearchParams = new URLSearchParams(currentSearchParams);
        nextSearchParams.set(SORT_COLUMN_SEARCH_PARAMETER, nextSortColumn);
        nextSearchParams.set(
          SORT_DIRECTION_SEARCH_PARAMETER,
          nextSortColumn === sortColumn
            ? OPPOSITE_SORT_DIRECTIONS[sortDirection]
            : DEFAULT_SORT_DIRECTION,
        );
        nextSearchParams.delete(CONVERSATION_PAGE_SEARCH_PARAMETER);
        return nextSearchParams;
      }),
    [setSearchParams, sortColumn, sortDirection],
  );
  const changeSearch = useCallback(
    (nextSearch: string) =>
      setSearchParams((currentSearchParams) => {
        const nextSearchParams = new URLSearchParams(currentSearchParams);
        if (nextSearch === "") {
          nextSearchParams.delete(SEARCH_TEXT_SEARCH_PARAMETER);
        } else {
          nextSearchParams.set(SEARCH_TEXT_SEARCH_PARAMETER, nextSearch);
        }
        nextSearchParams.delete(CONVERSATION_PAGE_SEARCH_PARAMETER);
        return nextSearchParams;
      }),
    [setSearchParams],
  );
  return { usageList, changePage, changeSort, changeSearch };
}
