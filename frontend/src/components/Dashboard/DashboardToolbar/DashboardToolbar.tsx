import { Flex, IconButton, Toolbar } from "src/components/design-system";
import { useStore, DashboardMode } from "src/store";

export function DashboardToolbar() {
  const mode = useStore((state) => state.dashboard.mode);
  const showChartBrowser = useStore((state) => state.dashboard.showChartBrowser);
  const setShowChartBrowser = useStore((state) => state.dashboard.setShowChartBrowser);
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
