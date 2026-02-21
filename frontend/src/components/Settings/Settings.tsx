import { useMemo } from "react";
import "src/css/index.css";
import { Connections } from "src/components/Connection/ConnectionsPage";
import { SophoTabs, type TabItem } from "src/components/SophoNavigationMenu";
import { Flex } from "src/components/design-system/Flex/Flex";
import { Heading } from "src/components/design-system";

export function Settings() {
  const tabItems = useMemo<TabItem[]>(
    () => [
      {
        id: "connections",
        label: "Connections",
        content: <Connections />,
      },
    ],
    []
  );

  return (
    <Flex
      flex="grow"
      paddingX="2xl"
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
