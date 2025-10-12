import SidebarStyles from "src/components/Sidebar/Sidebar.module.css";
import { NavLink } from "react-router";
import { APP_ROUTES } from "src/constants/app_routes";
import * as NavigationMenu from "@radix-ui/react-navigation-menu";

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
                <span className="material-symbols-rounded">home</span>
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.NOTEBOOKS} className={SidebarStyles.link}>
                <span className="material-symbols-rounded">book_2</span>
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
          <NavigationMenu.Item className={SidebarStyles.navItem}>
            <NavigationMenu.Link asChild>
              <NavLink to={APP_ROUTES.SETTINGS} className={SidebarStyles.link}>
                <span className="material-symbols-rounded">settings</span>
              </NavLink>
            </NavigationMenu.Link>
          </NavigationMenu.Item>
        </NavigationMenu.List>
      </NavigationMenu.Root>
    </div>
  );
}
