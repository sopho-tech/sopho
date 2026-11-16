import { useUsageByKind } from "src/api/ai_usage";
import { UsageByWorkChart } from "src/components/Settings/AIUsage/UsageByWorkChart";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { useUsageRange } from "src/components/Settings/AIUsage/hooks";

export function UsageByWork() {
  const byKind = useUsageByKind(useUsageRange());

  if (byKind.error) {
    return <UsageFailure failure={byKind.error} />;
  }
  if (!byKind.data) {
    return <UsageLoading />;
  }
  return <UsageByWorkChart byKind={byKind.data} />;
}
