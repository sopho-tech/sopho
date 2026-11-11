import { useMemo } from "react";
import { Outlet } from "react-router";
import {
  SectionLayout,
  SectionNavigationItem,
} from "src/components/design-system";
import { usePlatformNavigationItems } from "src/components/Platform/hooks";
import { useTenantPermission } from "src/api/tenant/queries";
import { APP_ROUTES } from "src/constants/app_routes";
import { PERMISSIONS } from "src/constants/permissions";
import { useTenantPath } from "src/utils/tenant";

export function Settings() {
  const toTenantPath = useTenantPath();
  const platformNavigationItems = usePlatformNavigationItems();
  const canInvite = useTenantPermission(PERMISSIONS.MEMBER_INVITE);
  const items = useMemo<SectionNavigationItem[]>(
    () => [
      {
        id: "connections",
        label: "Connections",
        to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS),
        matchPrefixes: [toTenantPath("/settings/connection")],
      },
      {
        id: "ai-configurations",
        label: "AI Configurations",
        to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.AI_CONFIGURATIONS),
      },
      {
        id: "tenant-management",
        label: "Tenant Management",
        to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.MEMBERS),
        subItems: canInvite
          ? [
              {
                id: "members",
                label: "Members",
                to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.MEMBERS),
              },
              {
                id: "invitations",
                label: "Invitations",
                to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.INVITATIONS),
              },
            ]
          : undefined,
      },
      ...platformNavigationItems,
    ],
    [toTenantPath, platformNavigationItems, canInvite],
  );

  return (
    <SectionLayout title="Settings" items={items}>
      <Outlet />
    </SectionLayout>
  );
}
