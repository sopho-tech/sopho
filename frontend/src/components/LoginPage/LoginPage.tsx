import { ApiService } from "src/utils/api_client";
import { useGoogleLogin } from "@react-oauth/google";
import { useMutation } from "@tanstack/react-query";
import { API_ENDPOINTS } from "src/constants/api_endpoints";

interface CodeResponse {
  code: string;
}

export default function LoginPage() {
  const mutation = useMutation({
    mutationFn: (codeResponse: CodeResponse) => {
      return ApiService.post({
        url: `${import.meta.env.VITE_API_HOSTNAME}${API_ENDPOINTS.AUTH.CREATE_SESSION}`,
        data: {
          provider: "google",
          code: codeResponse.code,
        },
        headers: {
          "Content-Type": "application/json",
        },
        credentials: true,
      });
    },
    onSuccess: () => {},
    onError: () => {},
  });

  const googleLogin = useGoogleLogin({
    flow: "auth-code",
    onSuccess: async (codeResponse) => {
      mutation.mutate(codeResponse);
    },
    onError: () => {},
  });

  return (
    <div>
      <div>
        <button onClick={() => googleLogin()}>Sign in with Google</button>
      </div>
    </div>
  );
}
