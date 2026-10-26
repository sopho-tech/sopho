import { useMemo } from "react";
import { usePlatformPermission } from "src/api/auth/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";

export type PlatformNavigationItem = {
  id: string;
  label: string;
  to: string;
};

export function usePlatformNavigationItems(): PlatformNavigationItem[] {
  const canManageUsers = usePlatformPermission(
    PLATFORM_PERMISSIONS.USER_MANAGE,
  );
  const canManageEmails = usePlatformPermission(
    PLATFORM_PERMISSIONS.EMAIL_MANAGE,
  );
  return useMemo(() => {
    const items: PlatformNavigationItem[] = [];
    if (canManageUsers) {
      items.push({
        id: "users",
        label: "Users",
        to: APP_ROUTES.PLATFORM_ROUTES.USERS,
      });
    }
    if (canManageEmails) {
      items.push({
        id: "emails",
        label: "Emails",
        to: APP_ROUTES.PLATFORM_ROUTES.EMAILS,
      });
    }
    return items;
  }, [canManageUsers, canManageEmails]);
}
