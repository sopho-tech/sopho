import { useMemo } from "react";
import { Outlet } from "react-router";
import {
  SectionLayout,
  SectionNavigationItem,
} from "src/components/design-system";
import { usePlatformNavigationItems } from "src/components/Platform/hooks";
import { APP_ROUTES } from "src/constants/app_routes";
import { useTenantPath } from "src/utils/tenant";

export function Settings() {
  const toTenantPath = useTenantPath();
  const platformNavigationItems = usePlatformNavigationItems();
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
        id: "members",
        label: "Tenant Management",
        to: toTenantPath(APP_ROUTES.SETTINGS_ROUTES.MEMBERS),
      },
      ...platformNavigationItems,
    ],
    [toTenantPath, platformNavigationItems],
  );

  return (
    <SectionLayout title="Settings" items={items}>
      <Outlet />
    </SectionLayout>
  );
}
