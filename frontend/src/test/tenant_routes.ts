import { APP_ROUTES } from "src/constants/app_routes";
import { TEST_TENANT_ID } from "src/test/test_data";
import { tenantApiUrl, tenantPath } from "src/utils/tenant";

export function tenantRoutePattern(path: string): string {
  return `${APP_ROUTES.TENANT}${path}`;
}

export function testTenantPath(path: string): string {
  return tenantPath(TEST_TENANT_ID, path);
}

export function testTenantApiUrl(endpoint: string): string {
  return tenantApiUrl(TEST_TENANT_ID, endpoint);
}
