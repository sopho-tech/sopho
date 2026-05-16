import { useMemo } from "react";
import * as NavigationMenu from "@radix-ui/react-navigation-menu";
import { NavLink, Outlet, useLocation } from "react-router";
import { Flex, Box, Text } from "src/components/design-system";
import styles from "src/components/Settings/Settings.module.css";
import { APP_ROUTES } from "src/constants/app_routes";

type SettingsItem = {
  id: string;
  label: string;
  to: string;
  matchPrefixes?: string[];
};

export function Settings() {
  const { pathname } = useLocation();
  const items = useMemo<SettingsItem[]>(
    () => [
      {
        id: "connections",
        label: "Connections",
        to: APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS,
        matchPrefixes: ["/settings/connection"],
      },
      {
        id: "ai-configurations",
        label: "AI Configurations",
        to: APP_ROUTES.SETTINGS_ROUTES.AI_CONFIGURATIONS,
      },
    ],
    [],
  );

  return (
    <Flex flex="grow" direction="row" overflow="hidden">
      <Flex
        as="aside"
        direction="column"
        paddingY="lg"
        paddingX="sm"
        gap="xs"
        borderRight="divider"
        width="15%"
        sx={{ backgroundColor: "var(--color-primary-100)" }}
      >
        <Box paddingX="sm" paddingBottom="xs">
          <Text fontSize="xs" color="darkGrey">
            Settings
          </Text>
        </Box>
        <NavigationMenu.Root orientation="vertical">
          <NavigationMenu.List className={styles.navList}>
            {items.map((item) => (
              <NavigationMenu.Item key={item.id}>
                <NavigationMenu.Link asChild>
                  <NavLink to={item.to}>
                    {({ isActive }) => {
                      const active =
                        isActive ||
                        (item.matchPrefixes?.some(
                          (prefix) =>
                            pathname === prefix ||
                            pathname.startsWith(`${prefix}/`),
                        ) ??
                          false);
                      return (
                        <Box
                          paddingX="sm"
                          paddingY="xs"
                          width="100%"
                          className={`${styles.navItem} ${active ? styles.navItemActive : ""}`}
                        >
                          <Text
                            fontSize="sm"
                            color={active ? "default" : "darkGrey"}
                          >
                            {item.label}
                          </Text>
                        </Box>
                      );
                    }}
                  </NavLink>
                </NavigationMenu.Link>
              </NavigationMenu.Item>
            ))}
          </NavigationMenu.List>
        </NavigationMenu.Root>
      </Flex>
      <Flex
        flex="grow"
        direction="column"
        paddingX="2xl"
        paddingY="lg"
        overflow="hidden"
      >
        <Outlet />
      </Flex>
    </Flex>
  );
}
