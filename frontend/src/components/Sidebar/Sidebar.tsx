import SidebarStyles from "src/components/Sidebar/Sidebar.module.css";
import { NavLink } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import * as NavigationMenu from "@radix-ui/react-navigation-menu";
import { IconButton } from "src/components/design-system";

export function Sidebar() {
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
                <IconButton
                  type="home"
                  backgroundColor="white"
                  iconColor="black"
                  tooltip={{ text: "Home", direction: "top" }}
                ></IconButton>
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.NOTEBOOKS} className={SidebarStyles.link}>
                <IconButton
                  type="book_2"
                  backgroundColor="white"
                  iconColor="black"
                  tooltip={{ text: "Notebooks", direction: "top" }}
                ></IconButton>
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.SETTINGS} className={SidebarStyles.link}>
                <IconButton
                  type="settings"
                  backgroundColor="white"
                  iconColor="black"
                  tooltip={{ text: "Settings", direction: "top" }}
                ></IconButton>
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
        </NavigationMenu.List>
      </NavigationMenu.Root>
    </div>
  );
}
