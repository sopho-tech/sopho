import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiError } from "src/api/dto";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES } from "src/constants/app_routes";
import { PlatformPermission } from "src/constants/platform_permissions";
import { ApiService } from "src/utils/api_client";

export type CreateSessionDto = {
  email: string;
  password: string;
};

export type UserDto = {
  id: string;
  username: string;
  email: string;
  full_name: string;
  platform_permissions: string[];
};

export type OidcProviderDto = {
  name: string;
  display_name: string;
};

export const authKeys = {
  all: ["auth"] as const,
  session: () => [...authKeys.all, "session"] as const,
  currentUser: () => [...authKeys.all, "currentUser"] as const,
  oidcProviders: () => [...authKeys.all, "oidcProviders"] as const,
};

const authApi = {
  isSessionValid: async (): Promise<boolean> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.AUTH.GET_SESSION,
      onlyBody: false,
    });
    return (response as Response).status === 200;
  },

  createSession: async (payload: CreateSessionDto) => {
    return ApiService.post({
      url: API_ENDPOINTS.AUTH.CREATE_SESSION,
      data: payload,
      headers: { "Content-Type": "application/json" },
    });
  },

  getCurrentUser: async (): Promise<UserDto> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.AUTH.GET_PROFILE_DETAILS,
      onlyBody: true,
    });
    return response as UserDto;
  },

  deleteSession: async (): Promise<void> => {
    await ApiService.delete({ url: API_ENDPOINTS.AUTH.DELETE_SESSION });
  },

  getOidcProviders: async (): Promise<OidcProviderDto[]> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.AUTH.OIDC_PROVIDERS,
      onlyBody: true,
    });
    return response as OidcProviderDto[];
  },

  refreshSessionTokens: async (): Promise<boolean> => {
    try {
      await ApiService.post({
        url: API_ENDPOINTS.AUTH.REFRESH_TOKEN,
        onlyBody: false,
      });
      return true;
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) {
        return false;
      }
      throw error;
    }
  },
};

export const refreshSessionTokens = authApi.refreshSessionTokens;

export const useSessionValid = () => {
  return useQuery({
    queryKey: authKeys.session(),
    queryFn: authApi.isSessionValid,
    retry: false,
    staleTime: 5 * 60 * 1000,
  });
};

export const useCreateSession = (callbacks?: {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}) => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: authApi.createSession,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: authKeys.session() });
      void queryClient.invalidateQueries({ queryKey: authKeys.currentUser() });
      callbacks?.onSuccess?.();
    },
    onError: (error) => {
      callbacks?.onError?.(error);
    },
  });
};

export const useCurrentUser = (enabled: boolean = true) => {
  return useQuery({
    queryKey: authKeys.currentUser(),
    queryFn: authApi.getCurrentUser,
    retry: false,
    staleTime: 5 * 60 * 1000,
    refetchOnMount: true,
    enabled,
  });
};

export const useSignedInUser = (): UserDto => {
  const currentUser = useCurrentUser();
  if (!currentUser.data) {
    throw new Error("The signed-in user is not loaded");
  }
  return currentUser.data;
};

export const usePlatformPermission = (
  permission: PlatformPermission,
): boolean => useSignedInUser().platform_permissions.includes(permission);

export const useOidcProviders = () => {
  return useQuery({
    queryKey: authKeys.oidcProviders(),
    queryFn: authApi.getOidcProviders,
    staleTime: Infinity,
  });
};

export const useDeleteSession = (callbacks?: {
  onError?: (error: Error) => void;
}) => {
  return useMutation({
    mutationFn: authApi.deleteSession,
    onSuccess: () => {
      window.location.replace(APP_ROUTES.SIGN_IN);
    },
    onError: (error) => {
      callbacks?.onError?.(error);
    },
  });
};
