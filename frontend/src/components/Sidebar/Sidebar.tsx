import SidebarStyles from "src/components/Sidebar/Sidebar.module.css";
import { NavLink, useLocation } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import * as NavigationMenu from "@radix-ui/react-navigation-menu";
import {
  IconButtonLink,
  IconButtonLinkState,
} from "src/components/design-system";
import logo from "src/assets/images/logo.svg";

const HOME_TOOLTIP = { text: "Home", direction: "right" } as const;
const NOTEBOOKS_TOOLTIP = { text: "Notebooks", direction: "right" } as const;
const SETTINGS_TOOLTIP = { text: "Settings", direction: "right" } as const;
const PROFILE_TOOLTIP = { text: "Profile", direction: "right" } as const;
const CONVERSATION_ANALYTICS_TOOLTIP = {
  text: "Conversational Analytics",
  direction: "right",
} as const;

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
            <NavLink to={APP_ROUTES.INDEX} className={SidebarStyles.link}>
              <img src={logo} alt="Logo" className={SidebarStyles.logo} />
            </NavLink>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.INDEX} className={SidebarStyles.link}>
                <IconButtonLink
                  type="home"
                  state={getState(APP_ROUTES.INDEX)}
                  tooltip={HOME_TOOLTIP}
                />
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.CANVASES} className={SidebarStyles.link}>
                <IconButtonLink
                  type="layers"
                  state={getState(APP_ROUTES.CANVASES)}
                  tooltip={NOTEBOOKS_TOOLTIP}
                />
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink
                to={APP_ROUTES.CONVERSATIONAL_ANALYTICS}
                className={SidebarStyles.link}
              >
                <IconButtonLink
                  type="bot"
                  state={getState(APP_ROUTES.CONVERSATIONAL_ANALYTICS)}
                  tooltip={CONVERSATION_ANALYTICS_TOOLTIP}
                />
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink
                to={APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS}
                className={SidebarStyles.link}
              >
                <IconButtonLink
                  type="settings"
                  state={getState(APP_ROUTES.SETTINGS)}
                  tooltip={SETTINGS_TOOLTIP}
                />
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.PROFILE} className={SidebarStyles.link}>
                <IconButtonLink
                  type="user"
                  state={getState(APP_ROUTES.PROFILE)}
                  tooltip={PROFILE_TOOLTIP}
                />
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
        </NavigationMenu.List>
      </NavigationMenu.Root>
    </div>
  );
}
