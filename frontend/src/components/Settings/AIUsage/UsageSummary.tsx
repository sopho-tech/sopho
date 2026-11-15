import { useUsageOverTime, useUsageSummary } from "src/api/ai_usage";
import { Flex } from "src/components/design-system";
import { UsageOverTimeChart } from "src/components/Settings/AIUsage/UsageOverTimeChart";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { UsageTiles } from "src/components/Settings/AIUsage/UsageTiles";
import { useUsageRange } from "src/components/Settings/AIUsage/hooks";

export function UsageSummary() {
  const range = useUsageRange();
  const summary = useUsageSummary(range);
  const overTime = useUsageOverTime(range);

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
