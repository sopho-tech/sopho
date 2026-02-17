import { useCallback, useMemo } from "react";
import HorizontalProgressBarStyles from "src/components/HorizontalProgressBar/HorizontalProgressBar.module.css";

export interface ProgressStep {
  id: string;
  label: string;
  isActive?: boolean;
  isCompleted?: boolean;
}

interface HorizontalProgressBarProps {
  steps: ProgressStep[];
  currentStepId: string;
  onStepClick?: (stepId: string) => void;
}

export function HorizontalProgressBar({
  steps,
  currentStepId,
  onStepClick,
}: HorizontalProgressBarProps) {
  const handleStepClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const stepId = (e.currentTarget as HTMLDivElement).dataset.stepId;
      if (stepId) onStepClick?.(stepId);
    },
    [onStepClick]
  );

  const stepStyle = useMemo(
    () => ({ cursor: onStepClick ? "pointer" : "default" }),
    [onStepClick]
  );

  return (
    <div className={HorizontalProgressBarStyles.progressContainer}>
      <div className={HorizontalProgressBarStyles.progressBar}>
        {steps.map((step, index) => {
          const isActive = step.id === currentStepId;
          const isCompleted =
            step.isCompleted ||
            steps.findIndex((s) => s.id === currentStepId) > index;

          return (
            <div
              key={step.id}
              className={`${HorizontalProgressBarStyles.progressStep} ${
                isActive ? HorizontalProgressBarStyles.active : ""
              } ${isCompleted ? HorizontalProgressBarStyles.completed : ""}`}
              data-step={index + 1}
              data-step-id={step.id}
              onClick={handleStepClick}
              style={stepStyle}
            >
              <div className={HorizontalProgressBarStyles.stepIndicator}>
                <span className={HorizontalProgressBarStyles.stepNumber}>
                  {index + 1}
                </span>
              </div>
              <div className={HorizontalProgressBarStyles.stepLabel}>
                {step.label}
              </div>
            </div>
          );
        })}
        <div className={HorizontalProgressBarStyles.progressLine}>
          <div
            className={`${HorizontalProgressBarStyles.progressFill} ${
              steps.findIndex((s) => s.id === currentStepId) > 0
                ? HorizontalProgressBarStyles.completed
                : ""
            }`}
          />
        </div>
      </div>
    </div>
  );
}
