import { useMemo } from "react";
import { usePlatformPermission } from "src/api/auth/queries";
import { SectionNavigationItem } from "src/components/design-system";
import { APP_ROUTES } from "src/constants/app_routes";
import { PLATFORM_PERMISSIONS } from "src/constants/platform_permissions";

export function usePlatformNavigationItems(): SectionNavigationItem[] {
  const canManageUsers = usePlatformPermission(
    PLATFORM_PERMISSIONS.USER_MANAGE,
  );
  const canCreateTenant = usePlatformPermission(
    PLATFORM_PERMISSIONS.TENANT_CREATE,
  );
  const canManageEmails = usePlatformPermission(
    PLATFORM_PERMISSIONS.EMAIL_MANAGE,
  );
  const canReadPlatformAiUsage = usePlatformPermission(
    PLATFORM_PERMISSIONS.PLATFORM_AI_USAGE_READ,
  );
  return useMemo(() => {
    const items: SectionNavigationItem[] = [];
    if (canManageUsers) {
      items.push({
        id: "users",
        label: "Users",
        to: APP_ROUTES.PLATFORM_ROUTES.USERS,
      });
    }
    if (canCreateTenant) {
      items.push({
        id: "tenants",
        label: "Tenants",
        to: APP_ROUTES.PLATFORM_ROUTES.TENANTS,
      });
    }
    if (canReadPlatformAiUsage) {
      items.push({
        id: "ai-usage",
        label: "AI Usage",
        to: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_SUMMARY,
        matchPrefixes: [APP_ROUTES.PLATFORM_ROUTES.AI_USAGE],
      });
    }
    if (canManageEmails) {
      items.push({
        id: "emails",
        label: "Emails",
        to: APP_ROUTES.PLATFORM_ROUTES.SENT_EMAILS,
        subItems: [
          {
            id: "sent-emails",
            label: "Sent",
            to: APP_ROUTES.PLATFORM_ROUTES.SENT_EMAILS,
          },
          {
            id: "email-templates",
            label: "Templates",
            to: APP_ROUTES.PLATFORM_ROUTES.EMAIL_TEMPLATES,
          },
        ],
      });
    }
    return items;
  }, [
    canManageUsers,
    canCreateTenant,
    canManageEmails,
    canReadPlatformAiUsage,
  ]);
}
