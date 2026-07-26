import { useCallback, useState } from "react";
import { PaginationState, Updater } from "@tanstack/react-table";
import classNames from "classnames";
import { useSearchParams } from "react-router";
import { useStore } from "src/store";
import { SEARCH_PARAMS } from "src/constants/app_routes";
import { useScrollToElement } from "src/utils/hooks";

const INITIAL_PAGE_SIZE = 20;

export function useCellContainerClassName(
  cellId: string,
  baseClassName: string,
  activeClassName: string
) {
  const activeCellId = useStore((state) => state.notebook.activeCellId);
  return classNames(cellId === activeCellId && activeClassName, baseClassName);
}

export function useScrollToCellFromUrl() {
  const [searchParams, setSearchParams] = useSearchParams();
  const cellId = searchParams.get(SEARCH_PARAMS.CELL);
  const setActiveCellId = useStore((state) => state.notebook.setActiveCellId);

  const handleScrolled = useCallback(() => {
    if (!cellId) {
      return;
    }
    setActiveCellId(cellId);
    setSearchParams(
      (params) => {
        params.delete(SEARCH_PARAMS.CELL);
        return params;
      },
      { replace: true }
    );
  }, [cellId, setActiveCellId, setSearchParams]);

  useScrollToElement(cellId, handleScrolled);
}

const INITIAL_PAGE_INDEX = 0;

export function useNotebooksPagination() {
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: INITIAL_PAGE_INDEX,
    pageSize: INITIAL_PAGE_SIZE,
  });

  const handlePaginationChange = (updaterOrValue: Updater<PaginationState>) => {
    setPagination((prev) =>
      typeof updaterOrValue === "function"
        ? updaterOrValue(prev)
        : updaterOrValue
    );
  };

  const handleChangePageSize = (newPageSize: string) => {
    setPagination(() => ({
      pageIndex: INITIAL_PAGE_INDEX,
      pageSize: Number(newPageSize),
    }));
  };

  const handlePageClick = (newPage: number) => {
    setPagination((prev) => ({ ...prev, pageIndex: newPage }));
  };

  return {
    pagination,
    handlePaginationChange,
    handleChangePageSize,
    handlePageClick,
  };
}
