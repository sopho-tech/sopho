import { Flex, IconButton, Toolbar } from "src/components/design-system";
import { DashboardMode, useDashboardStore } from "../store";

export function DashboardToolbar() {
  const { mode, showChartBrowser, setShowChartBrowser } = useDashboardStore();
  if (mode == DashboardMode.VIEWING) {
    return null;
  }
  return (
    <Toolbar orientation="vertical" asChild>
      <Flex
        borderRadius="lg"
        backgroundColor="grey"
        direction="column"
        gap="sm"
        paddingX="2xs"
        paddingY="2xs"
        alignSelf="flex-start"
        shadow="xs"
      >
        <IconButton
          type="search"
          backgroundColor="transparent"
          iconColor={showChartBrowser ? "accent" : "black"}
          onClick={() => setShowChartBrowser(!showChartBrowser)}
          tooltip={{
            text: "search chart cells",
            direction: "top",
          }}
        />
        <IconButton
          type="sparkles"
          backgroundColor="transparent"
          iconColor="black"
          onClick={() => {}}
          tooltip={{
            text: "auto layout dashboard",
            direction: "top",
          }}
        />
      </Flex>
    </Toolbar>
  );
}
