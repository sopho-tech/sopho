import { Pagination } from "src/components/design-system";
import { FIRST_PAGE, SINGLE_PAGE_COUNT } from "src/constants/pagination";

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
  if (
    listPage.total_pages <= SINGLE_PAGE_COUNT &&
    listPage.page === FIRST_PAGE
  ) {
    return null;
  }
  return (
    <Pagination
      totalPages={listPage.total_pages}
      currentPage={listPage.page}
      pageSize={listPage.page_size}
      totalItems={listPage.total}
      showRowsPerPage={false}
      onChangePageSize={refusePageSizeChange}
      onPageClick={onPageChange}
    />
  );
}
