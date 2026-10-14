import { Navigate, Outlet } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { useSessionValid } from "src/api/auth_api";

export function ProtectedRoute() {
  const { data: isAuthenticated, isLoading } = useSessionValid();

  if (isLoading) return null;
  return isAuthenticated ? (
    <Outlet />
  ) : (
    <Navigate to={APP_ROUTES.SIGN_IN} replace />
  );
}
