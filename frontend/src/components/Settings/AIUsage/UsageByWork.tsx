import { useUsageByKind } from "src/api/ai_usage";
import { PieChart } from "src/components/Chart/PieChart";
import { Box, Flex, Heading } from "src/components/design-system";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import {
  KIND_CHART_CATEGORY,
  KIND_CHART_VALUE,
  PIE_CHART_HEIGHT,
} from "src/components/Settings/AIUsage/constants";
import { useUsageRange } from "src/components/Settings/AIUsage/hooks";

export function UsageByWork() {
  const byKind = useUsageByKind(useUsageRange());

  if (byKind.error) {
    return <UsageFailure failure={byKind.error} />;
  }
  if (!byKind.data) {
    return <UsageLoading />;
  }
  return (
    <Flex
      as="section"
      direction="column"
      gap="sm"
      paddingX="md"
      paddingY="md"
      border="default"
      borderRadius="lg"
      aria-label="Tokens by kind of work"
    >
      <Heading accessibilityLevel={3} size="lg" textColor="black">
        Tokens by kind of work
      </Heading>
      <Box height={PIE_CHART_HEIGHT}>
        <PieChart
          category={KIND_CHART_CATEGORY}
          value={KIND_CHART_VALUE}
          data={byKind.data.kinds}
        />
      </Box>
    </Flex>
  );
}
