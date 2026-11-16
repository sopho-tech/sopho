import { usePlatformUsageByKind } from "src/api/platform_ai_usage";
import { UsageByWorkChart } from "src/components/Settings/AIUsage/UsageByWorkChart";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { useUsageRange } from "src/components/Settings/AIUsage/hooks";

export function PlatformUsageByWork() {
  const { lastDayCount } = useUsageRange();
  const byKind = usePlatformUsageByKind(lastDayCount);

  if (byKind.error) {
    return <UsageFailure failure={byKind.error} />;
  }
  if (!byKind.data) {
    return <UsageLoading />;
  }
  return <UsageByWorkChart byKind={byKind.data} />;
}
