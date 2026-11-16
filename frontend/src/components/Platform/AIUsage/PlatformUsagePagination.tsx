import { Pagination } from "src/components/design-system";

type PlatformUsagePaginationProps = {
  totalPages: number;
  page: number;
  pageSize: number;
  totalItems: number;
  onChangePage: (page: number) => void;
};

function ignorePageSizeChange() {}

export function PlatformUsagePagination({
  totalPages,
  page,
  pageSize,
  totalItems,
  onChangePage,
}: PlatformUsagePaginationProps) {
  return (
    <Pagination
      totalPages={totalPages}
      currentPage={page}
      pageSize={pageSize}
      totalItems={totalItems}
      onChangePageSize={ignorePageSizeChange}
      onPageClick={onChangePage}
      showRowsPerPage={false}
    />
  );
}
