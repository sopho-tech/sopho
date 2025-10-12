import "src/css/index.css";
import { useState } from "react";
import styles from "src/components/SophoNavigationMenu/SophoTabs.module.css";
import * as NavigationMenu from "@radix-ui/react-navigation-menu";

export type TabItem = {
  id: string;
  label: string;
  content: React.ReactNode;
};

export type SophoTabsProps = {
  items: TabItem[];
  defaultActiveItem?: string;
  className?: string;
  onTabChange?: (activeItem: string) => void;
};

export function SophoTabs({
  items,
  defaultActiveItem,
  className = "",
  onTabChange,
}: SophoTabsProps) {
  const [activeTab, setActiveTab] = useState(
    defaultActiveItem || items[0]?.id || ""
  );

  const handleTabChange = (itemId: string) => {
    setActiveTab(itemId);
    onTabChange?.(itemId);
  };

  const activeItem = items.find((item) => item.id === activeTab);

  return (
    <div className={`${styles.navigationContainer} ${className}`}>
      <NavigationMenu.Root className={styles.navigationMenu}>
        <NavigationMenu.List className={styles.navList}>
          {items.map((item) => (
            <NavigationMenu.Item
              key={item.id}
              className={activeTab === item.id ? styles.active : ""}
            >
              <NavigationMenu.Link asChild>
                <button
                  onClick={() => handleTabChange(item.id)}
                  className={styles.navButton}
                >
                  {item.label}
                </button>
              </NavigationMenu.Link>
            </NavigationMenu.Item>
          ))}
        </NavigationMenu.List>
      </NavigationMenu.Root>
      <div className={styles.content}>{activeItem?.content}</div>
    </div>
  );
}
