import { UsageOverTimeDto } from "src/api/ai_usage";
import { BarChart } from "src/components/Chart/BarChart";
import { Box, Flex, Heading } from "src/components/design-system";
import {
  CHART_HEIGHT,
  TOKENS_PER_DAY_SERIES,
  TOKENS_PER_DAY_X_AXIS,
} from "src/components/Settings/AIUsage/constants";

type UsageOverTimeChartProps = {
  overTime: UsageOverTimeDto;
};

export function UsageOverTimeChart({ overTime }: UsageOverTimeChartProps) {
  return (
    <Flex
      as="section"
      direction="column"
      gap="sm"
      paddingX="md"
      paddingY="md"
      border="default"
      borderRadius="lg"
      aria-label="Tokens per day"
    >
      <Heading accessibilityLevel={3} size="lg" textColor="black">
        Tokens per day
      </Heading>
      <Box height={CHART_HEIGHT}>
        <BarChart
          xAxis={TOKENS_PER_DAY_X_AXIS}
          series={TOKENS_PER_DAY_SERIES}
          data={overTime.days}
          stacked
        />
      </Box>
    </Flex>
  );
}
