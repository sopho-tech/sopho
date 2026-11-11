import * as NavigationMenu from "@radix-ui/react-navigation-menu";
import { NavLink, useLocation } from "react-router";
import { Box } from "src/components/design-system/Box";
import { Flex } from "src/components/design-system/Flex";
import { Text } from "src/components/design-system/Text";
import styles from "src/components/design-system/SectionLayout/SectionLayout.module.css";

export type SectionNavigationSubItem = {
  id: string;
  label: string;
  to: string;
};

export type SectionNavigationItem = {
  id: string;
  label: string;
  to: string;
  matchPrefixes?: string[];
  subItems?: SectionNavigationSubItem[];
};

export type SectionLayoutProps = {
  title: string;
  items: SectionNavigationItem[];
  children: React.ReactNode;
};

function isPathUnderPrefix(pathname: string, prefix: string) {
  return pathname === prefix || pathname.startsWith(`${prefix}/`);
}

type SectionNavigationLinkProps = {
  label: string;
  to: string;
  isActive: boolean;
  variant: "item" | "group" | "subItem";
};

function SectionNavigationLink({
  label,
  to,
  isActive,
  variant,
}: SectionNavigationLinkProps) {
  const isHighlighted = isActive && variant !== "group";
  return (
    <NavigationMenu.Item>
      <NavigationMenu.Link asChild active={isHighlighted}>
        <NavLink to={to}>
          <Box
            paddingX="sm"
            paddingY={variant === "subItem" ? "2xs" : "xs"}
            className={`${styles.navItem} ${variant === "subItem" ? styles.navSubItem : ""} ${isHighlighted ? styles.navItemActive : ""}`}
          >
            <Text
              fontSize={variant === "subItem" ? "xs" : "sm"}
              color={isActive ? "default" : "darkGrey"}
            >
              {label}
            </Text>
          </Box>
        </NavLink>
      </NavigationMenu.Link>
    </NavigationMenu.Item>
  );
}

type SectionNavigationEntryProps = {
  item: SectionNavigationItem;
  pathname: string;
};

function SectionNavigationEntry({
  item,
  pathname,
}: SectionNavigationEntryProps) {
  if (!item.subItems) {
    const prefixes = [item.to, ...(item.matchPrefixes ?? [])];
    return (
      <SectionNavigationLink
        label={item.label}
        to={item.to}
        isActive={prefixes.some((prefix) =>
          isPathUnderPrefix(pathname, prefix),
        )}
        variant="item"
      />
    );
  }
  return (
    <>
      <SectionNavigationLink
        label={item.label}
        to={item.to}
        isActive={item.subItems.some((subItem) =>
          isPathUnderPrefix(pathname, subItem.to),
        )}
        variant="group"
      />
      {item.subItems.map((subItem) => (
        <SectionNavigationLink
          key={subItem.id}
          label={subItem.label}
          to={subItem.to}
          isActive={isPathUnderPrefix(pathname, subItem.to)}
          variant="subItem"
        />
      ))}
    </>
  );
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
              <SectionNavigationEntry
                key={item.id}
                item={item}
                pathname={pathname}
              />
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
