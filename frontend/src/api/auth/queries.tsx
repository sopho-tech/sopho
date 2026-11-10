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

export type ProviderIcon = "GOOGLE" | "GENERIC";

export type OidcProviderDto = {
  name: string;
  display_name: string;
  icon: ProviderIcon;
};

export type SignInOptionsDto = {
  providers: OidcProviderDto[];
  password_reset_available: boolean;
};

export type PasswordResetRequestedDto = {
  message: string;
};

export type SignInProviderDto = {
  name: string;
  display_name: string;
  connected: boolean;
  email: string | null;
  can_disconnect: boolean;
  disconnect_blocked_reason: string | null;
};

export type SignInMethodsDto = {
  has_password: boolean;
  providers: SignInProviderDto[];
};

export type ChangePasswordDto = {
  currentPassword: string;
  newPassword: string;
};

export type PasswordResetDto = {
  token: string;
  newPassword: string;
};

export const authKeys = {
  all: ["auth"] as const,
  session: () => [...authKeys.all, "session"] as const,
  currentUser: () => [...authKeys.all, "currentUser"] as const,
  signInOptions: () => [...authKeys.all, "signInOptions"] as const,
  signInMethods: () => [...authKeys.all, "signInMethods"] as const,
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

  getSignInMethods: async (): Promise<SignInMethodsDto> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.AUTH.SIGN_IN_METHODS,
      onlyBody: true,
    });
    return response as SignInMethodsDto;
  },

  changePassword: async ({
    currentPassword,
    newPassword,
  }: ChangePasswordDto) => {
    await ApiService.put({
      url: API_ENDPOINTS.AUTH.CHANGE_PASSWORD,
      data: { current_password: currentPassword, new_password: newPassword },
      headers: { "Content-Type": "application/json" },
      onlyBody: false,
    });
  },

  disconnectProvider: async (providerName: string) => {
    await ApiService.delete({
      url: API_ENDPOINTS.AUTH.OIDC_IDENTITY.replace(
        ":provider_name",
        providerName,
      ),
    });
  },

  getSignInOptions: async (): Promise<SignInOptionsDto> => {
    const response = await ApiService.get({
      url: API_ENDPOINTS.AUTH.SIGN_IN_OPTIONS,
      onlyBody: true,
    });
    return response as SignInOptionsDto;
  },

  requestPasswordReset: async (
    email: string,
  ): Promise<PasswordResetRequestedDto> => {
    const response = await ApiService.post({
      url: API_ENDPOINTS.AUTH.PASSWORD_RESET_REQUEST,
      data: { email },
      headers: { "Content-Type": "application/json" },
    });
    return response as PasswordResetRequestedDto;
  },

  resetPassword: async ({ token, newPassword }: PasswordResetDto) => {
    await ApiService.post({
      url: API_ENDPOINTS.AUTH.PASSWORD_RESET,
      data: { token, new_password: newPassword },
      headers: { "Content-Type": "application/json" },
      onlyBody: false,
    });
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

export const connectProviderUrl = (providerName: string): string =>
  API_ENDPOINTS.AUTH.OIDC_CONNECT.replace(":provider_name", providerName);

export const useSignInMethods = () => {
  return useQuery({
    queryKey: authKeys.signInMethods(),
    queryFn: authApi.getSignInMethods,
  });
};

export const useChangePassword = () => {
  return useMutation({ mutationFn: authApi.changePassword });
};

export const useDisconnectProvider = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: authApi.disconnectProvider,
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: authKeys.signInMethods() }),
  });
};

export const useSignInOptions = () => {
  return useQuery({
    queryKey: authKeys.signInOptions(),
    queryFn: authApi.getSignInOptions,
    staleTime: Infinity,
  });
};

export const useRequestPasswordReset = () => {
  return useMutation({ mutationFn: authApi.requestPasswordReset });
};

export const useResetPassword = () => {
  return useMutation({ mutationFn: authApi.resetPassword });
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
