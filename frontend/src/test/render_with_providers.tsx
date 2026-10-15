import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryRouter,
  RouterProvider,
  type RouteObject,
} from "react-router";
import { render } from "vitest-browser-react";
import { useStore } from "src/store";

const initialStoreState = useStore.getState();

type RenderWithProvidersOptions = {
  routes: RouteObject[];
  initialPath: string;
};

export async function renderWithProviders({
  routes,
  initialPath,
}: RenderWithProvidersOptions) {
  useStore.setState(initialStoreState, true);
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 0 },
      mutations: { retry: false },
    },
  });
  const router = createMemoryRouter(routes, { initialEntries: [initialPath] });
  const screen = await render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { screen, router, queryClient };
}
