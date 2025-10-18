import * as Progress from "@radix-ui/react-progress";
import ProgressStyles from "./SophoProgress.module.css";

const MIN_PROGRESS_VALUE = 15;

type SophoProgressProps = {
  max?: number;
  total: number;
  completed: number;
};

export function SophoProgress({
  max = 100,
  total,
  completed,
}: SophoProgressProps) {
  const progressValue = Math.max(
    Math.round((completed / total) * 100.0),
    MIN_PROGRESS_VALUE
  );
  return (
    <div className={ProgressStyles.container}>
      <Progress.Root
        className={ProgressStyles.progressRoot}
        value={progressValue}
        max={max}
      >
        <Progress.Indicator
          className={ProgressStyles.progressIndicator}
          style={{ "--progress": `${progressValue}%` } as React.CSSProperties}
        />
      </Progress.Root>
      <div className={ProgressStyles.counterContainer}>
        {completed} / {total} completed
      </div>
    </div>
  );
}
