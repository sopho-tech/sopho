import { useLocation } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import { CollapsibleSidebar } from "src/components/design-system";
import { TenantSwitcher } from "src/components/Sidebar/TenantSwitcher";
import { useTenantPath } from "src/utils/tenant";
import { useStore } from "src/store";

const withoutTrailingSlash = (path: string) => path.replace(/\/$/, "");

export function Sidebar() {
  const location = useLocation();
  const toTenantPath = useTenantPath();
  const isPinned = useStore((state) => state.sidebar.isPinned);
  const setIsPinned = useStore((state) => state.sidebar.setIsPinned);

  const isActive = (path: string) => {
    const currentPath = withoutTrailingSlash(location.pathname);
    const targetPath = withoutTrailingSlash(toTenantPath(path));
    if (path === APP_ROUTES.INDEX) {
      return currentPath === targetPath;
    }
    return currentPath.startsWith(targetPath);
  };

  return (
    <CollapsibleSidebar
      isPinned={isPinned}
      onPinnedChange={setIsPinned}
      header={<TenantSwitcher />}
    >
      <CollapsibleSidebar.Navigation>
        <CollapsibleSidebar.Link
          to={toTenantPath(APP_ROUTES.INDEX)}
          icon="home"
          label="Home"
          isActive={isActive(APP_ROUTES.INDEX)}
        />
        <CollapsibleSidebar.Link
          to={toTenantPath(APP_ROUTES.CANVASES)}
          icon="layers"
          label="Notebooks"
          isActive={isActive(APP_ROUTES.CANVASES)}
        />
        <CollapsibleSidebar.Link
          to={toTenantPath(APP_ROUTES.CONVERSATIONAL_ANALYTICS)}
          icon="bot"
          label="Conversational Analytics"
          isActive={isActive(APP_ROUTES.CONVERSATIONAL_ANALYTICS)}
        />
      </CollapsibleSidebar.Navigation>
      <CollapsibleSidebar.Footer>
        <CollapsibleSidebar.Navigation>
          <CollapsibleSidebar.Link
            to={toTenantPath(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS)}
            icon="settings"
            label="Settings"
            isActive={isActive(APP_ROUTES.SETTINGS)}
          />
          <CollapsibleSidebar.Link
            to={toTenantPath(APP_ROUTES.PROFILE)}
            icon="user"
            label="Profile"
            isActive={isActive(APP_ROUTES.PROFILE)}
          />
        </CollapsibleSidebar.Navigation>
      </CollapsibleSidebar.Footer>
    </CollapsibleSidebar>
  );
}
