import { useCallback } from "react";
import { Navigate, Outlet, useNavigate } from "react-router";
import { Flex, IconButton, SectionLayout } from "src/components/design-system";
import {
  EXIT_PLATFORM_LABEL,
  PLATFORM_AREA_TITLE,
} from "src/components/Platform/constants";
import { usePlatformNavigationItems } from "src/components/Platform/hooks";
import { APP_ROUTES } from "src/constants/app_routes";
import { useStore } from "src/store";
import { tenantPath } from "src/utils/tenant";

const EXIT_PLATFORM_TOOLTIP = { text: EXIT_PLATFORM_LABEL };

export function PlatformRoute() {
  const items = usePlatformNavigationItems();
  const navigate = useNavigate();
  const lastTenantId = useStore((state) => state.tenant.lastTenantId);
  const handleExitPlatform = useCallback(
    () =>
      void navigate(
        lastTenantId === null
          ? APP_ROUTES.INDEX
          : tenantPath(lastTenantId, APP_ROUTES.INDEX),
      ),
    [navigate, lastTenantId],
  );

  if (items.length === 0) {
    return <Navigate to={APP_ROUTES.INDEX} replace />;
  }

  return (
    <Flex direction="row" height="100vh" width="100vw">
      <SectionLayout
        title={PLATFORM_AREA_TITLE}
        titleIcon="shield_user"
        titleAction={
          <IconButton
            type="logout"
            backgroundColor="default"
            iconColor="grey"
            iconSize="md"
            iconMirrored
            tooltip={EXIT_PLATFORM_TOOLTIP}
            aria-label={EXIT_PLATFORM_LABEL}
            onClick={handleExitPlatform}
          />
        }
        items={items}
      >
        <Outlet />
      </SectionLayout>
    </Flex>
  );
}

export function PlatformIndexRedirect() {
  const [firstItem] = usePlatformNavigationItems();
  return <Navigate to={firstItem.to} replace />;
}
