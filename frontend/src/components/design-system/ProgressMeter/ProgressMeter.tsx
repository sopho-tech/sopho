import classNames from "classnames";
import { useMemo } from "react";
import styles from "src/components/design-system/ProgressMeter/ProgressMeter.module.css";

const FULL_PERCENTAGE = 100;

export type ProgressMeterTone = "green" | "yellow" | "red";
export type ProgressMeterWidth = "compact" | "full";

export type ProgressMeterProps = Omit<
  React.HTMLAttributes<HTMLSpanElement>,
  "children"
> & {
  percentage: number;
  tone: ProgressMeterTone;
  width?: ProgressMeterWidth;
  "aria-label": string;
  ref?: React.Ref<HTMLSpanElement>;
};

export function ProgressMeter({
  percentage,
  tone,
  width = "compact",
  "aria-label": ariaLabel,
  className,
  ref,
  ...restProps
}: ProgressMeterProps) {
  const percentageText = `${percentage}%`;
  const fillStyle = useMemo(
    () =>
      ({
        "--progress-meter-percentage": percentageText,
      }) as React.CSSProperties,
    [percentageText],
  );

  return (
    <span
      ref={ref}
      className={classNames(styles.progressMeter, styles[width], className)}
      {...restProps}
    >
      <span
        role="meter"
        aria-label={ariaLabel}
        aria-valuemin={0}
        aria-valuemax={FULL_PERCENTAGE}
        aria-valuenow={Math.min(percentage, FULL_PERCENTAGE)}
        aria-valuetext={percentageText}
        className={styles.track}
      >
        <span
          className={classNames(styles.fill, styles[tone])}
          style={fillStyle}
        />
      </span>
      <span className={styles.percentage} aria-hidden="true">
        {percentageText}
      </span>
    </span>
  );
}
