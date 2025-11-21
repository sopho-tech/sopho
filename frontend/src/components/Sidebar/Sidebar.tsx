import SidebarStyles from "src/components/Sidebar/Sidebar.module.css";
import { NavLink, useLocation } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import * as NavigationMenu from "@radix-ui/react-navigation-menu";
import {
  IconButtonLink,
  IconButtonLinkState,
} from "src/components/design-system";

export function Sidebar() {
  const location = useLocation();

  const getState = (path: string) => {
    if (path === APP_ROUTES.INDEX) {
      return location.pathname === APP_ROUTES.INDEX
        ? IconButtonLinkState.ACTIVE
        : IconButtonLinkState.INACTIVE;
    }
    return location.pathname.startsWith(path)
      ? IconButtonLinkState.ACTIVE
      : IconButtonLinkState.INACTIVE;
  };

  return (
    <div className={SidebarStyles.sidebar}>
      <NavigationMenu.Root>
        <NavigationMenu.List className={SidebarStyles.navItems}>
          <NavigationMenu.Item className={SidebarStyles.companyLogo}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.INDEX} className={SidebarStyles.link}>
                <span>S</span>
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.INDEX} className={SidebarStyles.link}>
                <IconButtonLink
                  type="home"
                  state={getState(APP_ROUTES.INDEX)}
                  tooltip={{ text: "Home", direction: "right" }}
                />
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.NOTEBOOKS} className={SidebarStyles.link}>
                <IconButtonLink
                  type="book"
                  state={getState(APP_ROUTES.NOTEBOOKS)}
                  tooltip={{ text: "Notebooks", direction: "right" }}
                />
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.SETTINGS} className={SidebarStyles.link}>
                <IconButtonLink
                  type="settings"
                  state={getState(APP_ROUTES.SETTINGS)}
                  tooltip={{ text: "Settings", direction: "right" }}
                />
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
        </NavigationMenu.List>
      </NavigationMenu.Root>
    </div>
  );
}
