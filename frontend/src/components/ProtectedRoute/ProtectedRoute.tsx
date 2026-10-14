import { Navigate, Outlet } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { useCurrentUser, useSessionValid } from "src/api/auth_api";
import { isUnauthorizedError } from "src/api/dto";

export function ProtectedRoute() {
  const { data: isAuthenticated, isLoading } = useSessionValid();
  const currentUser = useCurrentUser(isAuthenticated === true);

  if (isLoading) return null;
  if (!isAuthenticated) {
    return <Navigate to={APP_ROUTES.SIGN_IN} replace />;
  }
  if (currentUser.isPending) return null;
  if (currentUser.isError) {
    if (isUnauthorizedError(currentUser.error)) return null;
    throw currentUser.error;
  }
  return <Outlet />;
}
