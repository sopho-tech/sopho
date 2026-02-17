import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";

export type CreateSessionDto = {
  email: string;
  password: string;
};

export type UserDto = {
  id: string;
  username: string;
  email: string;
  full_name: string;
};

export const authKeys = {
  all: ["auth"] as const,
  session: () => [...authKeys.all, "session"] as const,
  currentUser: () => [...authKeys.all, "currentUser"] as const,
};

export const authApi = {
  isSessionValid: async (): Promise<boolean> => {
    const url = API_ENDPOINTS.AUTH.GET_SESSION;
    const response = await ApiService.get({ url, onlyBody: false });
    return (response as Response).status === 200;
  },
  createSession: async (payload: CreateSessionDto) => {
    const url = API_ENDPOINTS.AUTH.CREATE_SESSION;
    return ApiService.post({
      url,
      data: payload,
      headers: {
        "Content-Type": "application/json",
      },
    });
  },
  getCurrentUser: async (): Promise<UserDto> => {
    const url = API_ENDPOINTS.AUTH.GET_PROFILE_DETAILS;
    const response = (await ApiService.get<UserDto>({
      url,
      onlyBody: true,
    })) as UserDto;
    return response;
  },
  deleteSession: async (): Promise<void> => {
    const url = API_ENDPOINTS.AUTH.DELETE_SESSION;
    const response = await fetch(url, {
      method: "DELETE",
      credentials: "include",
    });

    if (!response.ok) {
      throw new Error(`Failed to delete session: ${response.statusText}`);
    }
  },
  refreshToken: async (): Promise<boolean> => {
    const url = API_ENDPOINTS.AUTH.REFRESH_TOKEN;
    const response = await ApiService.post({ url, onlyBody: false });
    return (response as Response).status === 200;
  },
};

export const useSessionValid = () => {
  return useQuery({
    queryKey: authKeys.session(),
    queryFn: authApi.isSessionValid,
    retry: false,
    staleTime: 5 * 60 * 1000, // 5 minutes
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
      // Invalidate the session query to refetch
      queryClient.invalidateQueries({
        queryKey: authKeys.session(),
      });

      // Invalidate the currentUser query to ensure it refetches when Profile component mounts
      queryClient.invalidateQueries({
        queryKey: authKeys.currentUser(),
      });

      // Call custom success callback if provided
      if (callbacks?.onSuccess) {
        callbacks.onSuccess();
      }
    },
    onError: (error) => {
      // Call custom error callback if provided
      if (callbacks?.onError) {
        callbacks.onError(error);
      }
    },
  });
};

export const useCurrentUser = () => {
  return useQuery({
    queryKey: authKeys.currentUser(),
    queryFn: authApi.getCurrentUser,
    retry: false,
    staleTime: 5 * 60 * 1000, // 5 minutes
    refetchOnMount: true, // Always refetch on mount to ensure fresh data after login
  });
};

export const useDeleteSession = (callbacks?: {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: authApi.deleteSession,
    onSuccess: () => {
      // Clear all auth-related queries
      queryClient.removeQueries({
        queryKey: authKeys.all,
      });

      // Call custom success callback if provided
      if (callbacks?.onSuccess) {
        callbacks.onSuccess();
      }
    },
    onError: (error) => {
      // Call custom error callback if provided
      if (callbacks?.onError) {
        callbacks.onError(error);
      }
    },
  });
};
