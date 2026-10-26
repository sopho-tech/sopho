import * as NavigationMenu from "@radix-ui/react-navigation-menu";
import { NavLink, Navigate, Outlet } from "react-router";
import { Box, Flex, Text } from "src/components/design-system";
import { usePlatformNavigationItems } from "src/components/Platform/hooks";
import {
  PLATFORM_ACTIVE_NAVIGATION_ITEM_STYLE,
  PLATFORM_ASIDE_STYLE,
  PLATFORM_NAVIGATION_ITEM_STYLE,
  PLATFORM_NAVIGATION_LIST_STYLE,
} from "src/components/Platform/constants";
import { APP_ROUTES } from "src/constants/app_routes";

export function PlatformLayout() {
  const items = usePlatformNavigationItems();

  if (items.length === 0) {
    return <Navigate to={APP_ROUTES.INDEX} replace />;
  }

  return (
    <Flex direction="row" height="100vh" width="100vw">
      <Flex
        as="aside"
        direction="column"
        paddingY="lg"
        paddingX="sm"
        gap="xs"
        borderRight="divider"
        width="15%"
        sx={PLATFORM_ASIDE_STYLE}
      >
        <Box paddingX="sm" paddingBottom="xs">
          <NavLink to={APP_ROUTES.INDEX}>
            <Text fontSize="xs" color="darkGrey">
              ← Back to Sopho
            </Text>
          </NavLink>
        </Box>
        <Box paddingX="sm" paddingBottom="xs">
          <Text fontSize="xs" color="darkGrey">
            Administration
          </Text>
        </Box>
        <NavigationMenu.Root orientation="vertical">
          <NavigationMenu.List style={PLATFORM_NAVIGATION_LIST_STYLE}>
            {items.map((item) => (
              <NavigationMenu.Item key={item.id}>
                <NavigationMenu.Link asChild>
                  <NavLink to={item.to}>
                    {({ isActive }) => (
                      <Box
                        paddingX="sm"
                        paddingY="xs"
                        width="100%"
                        sx={
                          isActive
                            ? PLATFORM_ACTIVE_NAVIGATION_ITEM_STYLE
                            : PLATFORM_NAVIGATION_ITEM_STYLE
                        }
                      >
                        <Text
                          fontSize="sm"
                          color={isActive ? "default" : "darkGrey"}
                        >
                          {item.label}
                        </Text>
                      </Box>
                    )}
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

export function PlatformIndexRedirect() {
  const [firstItem] = usePlatformNavigationItems();
  if (!firstItem) {
    return <Navigate to={APP_ROUTES.INDEX} replace />;
  }
  return <Navigate to={firstItem.to} replace />;
}
