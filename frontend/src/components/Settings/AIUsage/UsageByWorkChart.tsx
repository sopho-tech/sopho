import { UsageByKindDto } from "src/api/ai_usage";
import { PieChart } from "src/components/Chart/PieChart";
import { Box, Flex, Heading } from "src/components/design-system";
import {
  KIND_CHART_CATEGORY,
  KIND_CHART_VALUE,
  PIE_CHART_HEIGHT,
} from "src/components/Settings/AIUsage/constants";

type UsageByWorkChartProps = {
  byKind: UsageByKindDto;
};

export function UsageByWorkChart({ byKind }: UsageByWorkChartProps) {
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
          data={byKind.kinds}
        />
      </Box>
    </Flex>
  );
}
