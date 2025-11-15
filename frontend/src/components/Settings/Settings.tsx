import "src/css/index.css";
import { Connections } from "src/components/Connection/ConnectionsPage";
import { SophoTabs, type TabItem } from "src/components/SophoNavigationMenu";
import { Flex } from "src/components/design-system/Flex/Flex";
import { Heading } from "src/components/design-system";

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
    <Flex
      flex="grow"
      paddingX="xs"
      paddingY="xs"
      marginTop="xs"
      marginBottom="xs"
      marginLeft="xs"
      marginRight="xs"
      direction="column"
    >
      <Flex marginBottom="lg">
        <Heading accessbilityLevel={1}>Settings</Heading>
      </Flex>
      <SophoTabs items={tabItems} defaultActiveItem="connections" />
    </Flex>
  );
}
