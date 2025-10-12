import "src/css/index.css";
import styles from "src/components/Settings/Settings.module.css";
import ContainerStyles from "src/css/container.module.css";
import { Connections } from "src/components/Connection/ConnectionsPage";
import { SophoTabs, type TabItem } from "src/components/SophoNavigationMenu";

export function Settings() {
  const tabItems: TabItem[] = [
    {
      id: "connections",
      label: "Connections",
      content: <Connections />,
    },
    {
      id: "permissions",
      label: "Permissions",
      content: <div>Permissions content goes here</div>,
    },
    {
      id: "security",
      label: "Security",
      content: <div>Security content goes here</div>,
    },
    {
      id: "alerts",
      label: "Alerts & Reports",
      content: <div>Alerts & Reports content goes here</div>,
    },
  ];

  return (
    <div className={`${ContainerStyles.container} ${styles.settingsContainer}`}>
      <h3>Settings</h3>
      <SophoTabs items={tabItems} defaultActiveItem="connections" />
    </div>
  );
}
