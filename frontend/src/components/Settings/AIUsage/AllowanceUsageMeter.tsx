import { AllowanceUsage } from "src/api/ai_usage";
import {
  ProgressMeter,
  ProgressMeterWidth,
  ToolTip,
} from "src/components/design-system";
import {
  ALLOWANCE_USAGE_LABEL,
  ALLOWANCE_USAGE_LEVEL_TONES,
} from "src/components/Settings/AIUsage/constants";

type AllowanceUsageMeterProps = {
  allowanceUsage: AllowanceUsage;
  width?: ProgressMeterWidth;
};

export function AllowanceUsageMeter({
  allowanceUsage,
  width,
}: AllowanceUsageMeterProps) {
  return (
    <ToolTip messageElement={allowanceUsage.tokens_label}>
      <ProgressMeter
        percentage={allowanceUsage.used_percentage}
        tone={ALLOWANCE_USAGE_LEVEL_TONES[allowanceUsage.level]}
        width={width}
        aria-label={ALLOWANCE_USAGE_LABEL}
      />
    </ToolTip>
  );
}
