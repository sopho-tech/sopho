import { useCallback } from "react";
import { Flex, IconButton, Toolbar } from "src/components/design-system";
import { useStore, DashboardMode } from "src/store";

const SEARCH_TOOLTIP = { text: "search chart cells", direction: "top" } as const;
const SPARKLES_TOOLTIP = { text: "auto layout dashboard", direction: "top" } as const;

export function DashboardToolbar() {
  const mode = useStore((state) => state.dashboard.mode);
  const showChartBrowser = useStore((state) => state.dashboard.showChartBrowser);
  const setShowChartBrowser = useStore((state) => state.dashboard.setShowChartBrowser);

  const handleSearchClick = useCallback(() => {
    setShowChartBrowser(!showChartBrowser);
  }, [showChartBrowser, setShowChartBrowser]);

  const handleSparklesClick = useCallback(() => {}, []);

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
          onClick={handleSearchClick}
          tooltip={SEARCH_TOOLTIP}
        />
        <IconButton
          type="sparkles"
          backgroundColor="transparent"
          iconColor="black"
          onClick={handleSparklesClick}
          tooltip={SPARKLES_TOOLTIP}
        />
      </Flex>
    </Toolbar>
  );
}
