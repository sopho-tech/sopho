import { useQuery } from "@tanstack/react-query";
import { ApiService } from "src/utils/api_client";
import { API_ENDPOINTS } from "src/constants/api_endpoints";

export const authKeys = {
  all: ["auth"] as const,
  session: () => [...authKeys.all, "session"] as const,
};

const authApi = {
  isSessionValid: async (): Promise<boolean> => {
    const url = `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.AUTH.GET_SESSION}`;
    try {
      const response = await ApiService.get({ url, onlyBody: false });
      return response.status === 200;
    } catch (error) {
      throw error;
    }
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
