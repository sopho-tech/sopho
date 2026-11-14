import { useState } from "react";
import {
  MutationCache,
  QueryCache,
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";
import { tenantKeys } from "src/api/tenant/queries";
import {
  isTenantSuspendedError,
  queryErrorHandler,
  mutationErrorHandler,
} from "src/utils/error_handler";

export const ReactQueryClientProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const [queryClient] = useState(() => {
    const refreshMembershipsWhenTenantIsSuspended = (error: unknown) => {
      if (isTenantSuspendedError(error)) {
        void createdQueryClient.invalidateQueries({
          queryKey: tenantKeys.memberships,
        });
      }
    };
    const createdQueryClient: QueryClient = new QueryClient({
      defaultOptions: {
        queries: {
          // With SSR, we usually want to set some default staleTime
          // above 0 to avoid refetching immediately on the client
          staleTime: 60 * 1000,
          retry: false,
        },
      },
      mutationCache: new MutationCache({
        onError: (error, variables, context, mutation) => {
          mutationErrorHandler(error, variables, context, mutation);
          refreshMembershipsWhenTenantIsSuspended(error);
        },
      }),
      queryCache: new QueryCache({
        onError: (error, query) => {
          queryErrorHandler(error, query);
          refreshMembershipsWhenTenantIsSuspended(error);
        },
      }),
    });
    return createdQueryClient;
  });
  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};
