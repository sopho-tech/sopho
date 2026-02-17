import { Mutation, Query, QueryKey } from "@tanstack/react-query";
import { authApi } from "src/api/auth_api";
import { ApiError } from "src/api/dto";
import { APP_ROUTES } from "src/constants/app_routes";

let isRefreshing = false;

let failedQueue: {
  query?: Query<unknown, unknown, unknown, QueryKey>;
  mutation?: Mutation<unknown, unknown, unknown, unknown>;
  variables?: unknown;
}[] = [];

const refreshAndRetry = async (
  query?: Query<unknown, unknown, unknown, QueryKey>,
  mutation?: Mutation<unknown, unknown, unknown, unknown>,
  variables?: unknown
) => {
  if (isRefreshing) {
    failedQueue.push({ query, mutation, variables });
  } else {
    isRefreshing = true;
    failedQueue.push({ query, mutation, variables });
    const result = await authApi.refreshToken();
    if (result) {
      failedQueue.forEach(({ query, mutation, variables }) => {
        if (query) {
          query.fetch();
        } else if (mutation) {
          mutation.execute(variables);
        }
      });
    } else {
      // Redirect to sign-in page when refresh fails
      window.location.href = APP_ROUTES.SIGN_IN;
    }
    isRefreshing = false;
    failedQueue = [];
  }
};

const errorHandler = (
  error: unknown,
  query?: Query<unknown, unknown, unknown, QueryKey>,
  mutation?: Mutation<unknown, unknown, unknown, unknown>,
  variables?: unknown
) => {
  const status = error instanceof Response ? error.status : undefined;
  if (error instanceof ApiError) {
    if (status === 401) {
      if (query) {
        refreshAndRetry(query);
      } else if (mutation) {
        refreshAndRetry(undefined, mutation, variables);
      }
    }
  }
};

export const queryErrorHandler = (
  error: unknown,
  query: Query<unknown, unknown, unknown, QueryKey>
) => {
  errorHandler(error, query, undefined, undefined);
};

export const mutationErrorHandler = (
  error: unknown,
  variables: unknown,
  _context: unknown,
  mutation: Mutation<unknown, unknown, unknown, unknown>
) => {
  errorHandler(error, undefined, mutation, variables);
};
