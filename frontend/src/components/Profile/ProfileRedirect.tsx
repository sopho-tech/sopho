import { Navigate, useLocation } from "react-router";
import { useMemberships } from "src/api/tenant/queries";
import { isUnauthorizedError } from "src/api/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { tenantPath } from "src/utils/tenant";

export function ProfileRedirect() {
  const memberships = useMemberships();
  const location = useLocation();

  if (memberships.isPending) return null;
  if (memberships.isError) {
    if (isUnauthorizedError(memberships.error)) return null;
    throw memberships.error;
  }

  const [firstMembership] = memberships.data;
  if (!firstMembership) {
    return <Navigate to={APP_ROUTES.ONBOARDING} replace />;
  }
  return (
    <Navigate
      to={`${tenantPath(firstMembership.tenant_id, APP_ROUTES.PROFILE)}${location.search}`}
      replace
    />
  );
}
