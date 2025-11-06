export type PaginatedDataResponse<T> = {
  data: T[];
  total_pages: number;
  page: number;
  page_size: number;
  total_items: number;
};

export type PaginatedData<T> = {
  data: T[];
  totalPages: number;
  page: number;
  pageSize: number;
  totalItems: number;
};
