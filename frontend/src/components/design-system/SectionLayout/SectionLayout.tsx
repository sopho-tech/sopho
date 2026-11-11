import * as NavigationMenu from "@radix-ui/react-navigation-menu";
import { NavLink, useLocation } from "react-router";
import { Box } from "src/components/design-system/Box";
import { Flex } from "src/components/design-system/Flex";
import { Text } from "src/components/design-system/Text";
import styles from "src/components/design-system/SectionLayout/SectionLayout.module.css";

export type SectionNavigationItem = {
  id: string;
  label: string;
  to: string;
  matchPrefixes?: string[];
};

export type SectionLayoutProps = {
  title: string;
  items: SectionNavigationItem[];
  children: React.ReactNode;
};

function isPathUnderPrefix(pathname: string, prefix: string) {
  return pathname === prefix || pathname.startsWith(`${prefix}/`);
}

export function SectionLayout({ title, items, children }: SectionLayoutProps) {
  const { pathname } = useLocation();

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
        className={styles.aside}
      >
        <Box paddingX="sm" paddingBottom="xs">
          <Text fontSize="xs" color="darkGrey">
            {title}
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
                        (item.matchPrefixes?.some((prefix) =>
                          isPathUnderPrefix(pathname, prefix),
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
        overflow="scrollY"
      >
        {children}
      </Flex>
    </Flex>
  );
}
