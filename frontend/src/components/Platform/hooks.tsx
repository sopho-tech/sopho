import { useMemo } from "react";
import { usePlatformPermission } from "src/api/auth/queries";
import { SectionNavigationItem } from "src/components/design-system";
import { APP_ROUTES } from "src/constants/app_routes";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";
import { useTenantPath } from "src/utils/tenant";

export function usePlatformNavigationItems(): SectionNavigationItem[] {
  const toTenantPath = useTenantPath();
  const canManageUsers = usePlatformPermission(
    PLATFORM_PERMISSIONS.USER_MANAGE,
  );
  const canCreateTenant = usePlatformPermission(
    PLATFORM_PERMISSIONS.TENANT_CREATE,
  );
  const canManageEmails = usePlatformPermission(
    PLATFORM_PERMISSIONS.EMAIL_MANAGE,
  );
  return useMemo(() => {
    const items: SectionNavigationItem[] = [];
    if (canManageUsers) {
      items.push({
        id: "users",
        label: "Global Users",
        to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.USERS),
      });
    }
    if (canCreateTenant) {
      items.push({
        id: "tenants",
        label: "Global Tenants",
        to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.TENANTS),
      });
    }
    if (canManageEmails) {
      items.push({
        id: "emails",
        label: "Emails",
        to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.EMAILS),
      });
    }
    return items;
  }, [toTenantPath, canManageUsers, canCreateTenant, canManageEmails]);
}
