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
  variables?: unknown,
) => {
  if (isRefreshing) {
    failedQueue.push({ query, mutation, variables });
  } else {
    isRefreshing = true;
    failedQueue.push({ query, mutation, variables });
    try {
      const refreshed = await authApi.refreshToken();
      if (refreshed) {
        failedQueue.forEach(({ query, mutation, variables }) => {
          if (query) {
            void query.fetch();
          } else if (mutation) {
            void mutation.execute(variables);
          }
        });
      } else {
        redirectToSignIn();
      }
    } catch {
      redirectToSignIn();
    } finally {
      isRefreshing = false;
      failedQueue = [];
    }
  }
};

const redirectToSignIn = () => {
  if (window.location.pathname !== APP_ROUTES.SIGN_IN) {
    window.location.href = APP_ROUTES.SIGN_IN;
  }
};

const errorHandler = (
  error: unknown,
  query?: Query<unknown, unknown, unknown, QueryKey>,
  mutation?: Mutation<unknown, unknown, unknown, unknown>,
  variables?: unknown,
) => {
  if (error instanceof ApiError) {
    const api_error = error as ApiError;
    if (api_error.status === 401) {
      if (query) {
        void refreshAndRetry(query);
      } else if (mutation) {
        void refreshAndRetry(undefined, mutation, variables);
      }
    }
  }
};

export const queryErrorHandler = (
  error: unknown,
  query: Query<unknown, unknown, unknown, QueryKey>,
) => {
  errorHandler(error, query, undefined, undefined);
};

export const mutationErrorHandler = (
  error: unknown,
  variables: unknown,
  _context: unknown,
  mutation: Mutation<unknown, unknown, unknown, unknown>,
) => {
  errorHandler(error, undefined, mutation, variables);
};
