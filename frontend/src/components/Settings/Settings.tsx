import { useMemo } from "react";
import "src/css/index.css";
import { Connections } from "src/components/Connection/ConnectionsPage";
import { SophoTabs, type TabItem } from "src/components/SophoNavigationMenu";
import { Flex } from "src/components/design-system/Flex/Flex";

export function Settings() {
  const tabItems = useMemo<TabItem[]>(
    () => [
      {
        id: "connections",
        label: "Connections",
        content: <Connections />,
      },
    ],
    [],
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
      <SophoTabs items={tabItems} defaultActiveItem="connections" />
    </Flex>
  );
}
