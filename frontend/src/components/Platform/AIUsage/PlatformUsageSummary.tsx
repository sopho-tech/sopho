import {
  usePlatformUsageOverTime,
  usePlatformUsageSummary,
} from "src/api/platform_ai_usage";
import { Flex } from "src/components/design-system";
import { UsageOverTimeChart } from "src/components/Settings/AIUsage/UsageOverTimeChart";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { UsageTiles } from "src/components/Settings/AIUsage/UsageTiles";
import { useUsageRange } from "src/components/Settings/AIUsage/hooks";

export function PlatformUsageSummary() {
  const { lastDayCount } = useUsageRange();
  const summary = usePlatformUsageSummary(lastDayCount);
  const overTime = usePlatformUsageOverTime(lastDayCount);

  const failure = summary.error ?? overTime.error;
  if (failure) {
    return <UsageFailure failure={failure} />;
  }
  if (!summary.data || !overTime.data) {
    return <UsageLoading />;
  }
  return (
    <Flex direction="column" gap="lg">
      <UsageTiles summary={summary.data} />
      <UsageOverTimeChart overTime={overTime.data} />
    </Flex>
  );
}
