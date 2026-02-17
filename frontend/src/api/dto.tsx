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

export type ApiErrorBody = {
  status?: number;
  code?: string;
  message?: string;
  details?: string;
};

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly body: ApiErrorBody,
    message?: string
  ) {
    super(
      message ?? body.message ?? body.message ?? `Request failed with ${status}`
    );
    this.name = "ApiError";
  }

  get code(): string | undefined {
    return this.body.code;
  }

  get message(): string {
    return this.body.message ?? this.body.message ?? "";
  }
}

