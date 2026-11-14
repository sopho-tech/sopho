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
  error_code?: string;
  message?: string;
  error?: string;
  details?: string;
  request_id?: string;
};

const FIRST_SERVER_ERROR_STATUS = 500;
const REQUEST_ID_LABEL = "Request ID";

export type ResourceCapabilities = {
  can_edit: boolean;
  can_delete: boolean;
  can_share: boolean;
};

function displayedErrorMessage(
  status: number,
  body: ApiErrorBody,
  message: string | undefined,
): string {
  const backendMessage =
    message ?? body.message ?? body.error ?? `Request failed with ${status}`;
  if (status < FIRST_SERVER_ERROR_STATUS || body.request_id === undefined) {
    return backendMessage;
  }
  return `${backendMessage} (${REQUEST_ID_LABEL}: ${body.request_id})`;
}

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly body: ApiErrorBody,
    message?: string,
  ) {
    super(displayedErrorMessage(status, body, message));
    this.name = "ApiError";
  }

  get code(): string | undefined {
    return this.body.code;
  }

  get errorCode(): string | undefined {
    return this.body.error_code;
  }

  get requestId(): string | undefined {
    return this.body.request_id;
  }
}

export const isUnauthorizedError = (error: unknown): boolean =>
  error instanceof ApiError && error.status === 401;

export enum ActorKind {
  HUMAN = "HUMAN",
  AI = "AI",
  SYSTEM = "SYSTEM",
}

export type AttributedUserDto = {
  user_id: string;
  full_name: string;
};

export type AttributionDto = {
  actor: ActorKind;
  user: AttributedUserDto | null;
};

export enum CreatedByFilter {
  ME = "me",
}
