import { useState } from "react";
import { PaginationState, Updater } from "@tanstack/react-table";

const INITIAL_PAGE_SIZE = 20;
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
