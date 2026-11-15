import { useState } from "react";
import {
  MutationCache,
  QueryCache,
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";
import { AI_CONFIGURATION_QUERY_SCOPE } from "src/api/ai_configuration";
import { tenantKeys } from "src/api/tenant/queries";
import {
  isPlatformAiAllowanceUsedError,
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
    const refreshAiStatusWhenThePlatformAiAllowanceIsUsed = (
      error: unknown,
    ) => {
      if (isPlatformAiAllowanceUsedError(error)) {
        void createdQueryClient.invalidateQueries({
          predicate: (query) =>
            query.queryKey.includes(AI_CONFIGURATION_QUERY_SCOPE),
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
          refreshAiStatusWhenThePlatformAiAllowanceIsUsed(error);
        },
      }),
      queryCache: new QueryCache({
        onError: (error, query) => {
          queryErrorHandler(error, query);
          refreshMembershipsWhenTenantIsSuspended(error);
          refreshAiStatusWhenThePlatformAiAllowanceIsUsed(error);
        },
      }),
    });
    return createdQueryClient;
  });
  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};
