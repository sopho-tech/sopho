import { Navigate, Outlet } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { useSessionValid } from "src/api/auth_api";
import LayoutStyles from "src/css/layout.module.css";
import { Sidebar } from "src/components/Sidebar/Sidebar";

export function ProtectedRoute() {
  const { data: isAuthenticated, isLoading } = useSessionValid();

  const component = (
    <div className={LayoutStyles.layout}>
      <Sidebar />
      <div className={LayoutStyles.outlet}>
        <Outlet />
      </div>
    </div>
  );

  if (isLoading) return null;
  return isAuthenticated ? (
    component
  ) : (
    <Navigate to={APP_ROUTES.SIGN_IN} replace />
  );
}
