import { useMemo } from "react";
import { Navigate, useLocation, useParams } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { useTenantPath } from "src/utils/tenant";

export function CanvasIndexRedirect() {
  const params = useParams();
  const toTenantPath = useTenantPath();
  const { search } = useLocation();
  const to = useMemo(
    () => ({
      pathname: toTenantPath(
        `${APP_ROUTES.CANVAS.replace(":id", params.id!)}/${APP_ROUTES.CANVAS_ROUTES.NOTEBOOK}`,
      ),
      search,
    }),
    [params.id, search, toTenantPath],
  );

  return <Navigate to={to} replace />;
}
