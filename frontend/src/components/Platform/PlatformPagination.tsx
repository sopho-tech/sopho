import { useCallback } from "react";
import { Pagination } from "src/components/design-system";
import { FIRST_PAGE } from "src/components/Platform/constants";

type PlatformListPage = {
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
};

type PlatformPaginationProps = {
  listPage: PlatformListPage;
  onPageChange: (page: number) => void;
};

function refusePageSizeChange(): never {
  throw new Error("Platform lists have a fixed page size");
}

export function PlatformPagination({
  listPage,
  onPageChange,
}: PlatformPaginationProps) {
  const handlePageClick = useCallback(
    (pageIndex: number) => onPageChange(pageIndex + FIRST_PAGE),
    [onPageChange],
  );

  if (listPage.total_pages <= FIRST_PAGE && listPage.page === FIRST_PAGE) {
    return null;
  }
  return (
    <Pagination
      totalPages={listPage.total_pages}
      currentPage={listPage.page - FIRST_PAGE}
      pageSize={listPage.page_size}
      totalItems={listPage.total}
      showRowsPerPage={false}
      onChangePageSize={refusePageSizeChange}
      onPageClick={handlePageClick}
    />
  );
}
