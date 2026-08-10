import { useMemo } from "react";
import { Navigate, useLocation, useParams } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";

export function CanvasIndexRedirect() {
  const params = useParams();
  const { search } = useLocation();
  const to = useMemo(
    () => ({
      pathname: `${APP_ROUTES.CANVAS.replace(":id", params.id!)}/${APP_ROUTES.CANVAS_ROUTES.NOTEBOOK}`,
      search,
    }),
    [params.id, search]
  );

  return <Navigate to={to} replace />;
}
