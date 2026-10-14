import { useCallback } from "react";
import { useParams } from "react-router";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { APP_ROUTES, TENANT_ROUTE_PARAMETER } from "src/constants/app_routes";

export function useTenantId(): string {
  const tenantId = useParams()[TENANT_ROUTE_PARAMETER];
  if (!tenantId) {
    throw new Error(
      `useTenantId was called outside a ${APP_ROUTES.TENANT} route`,
    );
  }
  return tenantId;
}

export function tenantPath(tenantId: string, path: string): string {
  return `${APP_ROUTES.TENANT.replace(`:${TENANT_ROUTE_PARAMETER}`, tenantId)}${path}`;
}

export function useTenantPath(): (path: string) => string {
  const tenantId = useTenantId();
  return useCallback((path: string) => tenantPath(tenantId, path), [tenantId]);
}

export function tenantApiUrl(tenantId: string, path: string): string {
  return `${API_ENDPOINTS.TENANT.SCOPED_BASE.replace(":tenant_id", tenantId)}${path}`;
}
