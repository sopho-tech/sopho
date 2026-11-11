import { Navigate, Outlet } from "react-router";
import { usePlatformNavigationItems } from "src/components/Platform/hooks";
import { APP_ROUTES } from "src/constants/app_routes";
import { useTenantPath } from "src/utils/tenant";

export function PlatformRoute() {
  const items = usePlatformNavigationItems();
  const toTenantPath = useTenantPath();

  if (items.length === 0) {
    return <Navigate to={toTenantPath(APP_ROUTES.SETTINGS)} replace />;
  }

  return <Outlet />;
}
