import {
  Label,
  TooltipContentProps,
  LegendProps,
  TooltipProps,
} from "recharts";
import {
  NameType,
  ValueType,
} from "recharts/types/component/DefaultTooltipContent";
import type { Props as LegendContentProps } from "recharts/types/component/DefaultLegendContent";
import { Flex } from "src/components/design-system";
import { getCSSVariable } from "src/utils/css_util";
import styles from "./ChartCommon.module.css";

export const CHART_MARGINS = {
  top: 50,
  right: 50,
  bottom: 0,
  left: 20,
};

export const TOOLTIP_STYLE = {
  border: "var(--border-default-medium)",
  borderRadius: "var(--border-radius-xl)",
  backgroundColor: "var(--color-grey-100)",
  boxShadow: "var(--shadow-sm)",
  padding: "var(--space-sm) var(--space-md)",
};

export const COMMON_LEGEND_PROPS: Partial<LegendProps> = {
  align: "center",
  verticalAlign: "bottom",
  iconSize: 16,
  inactiveColor: "#ccc",
  layout: "horizontal",
};

export const renderToolTipContent = (
  props: TooltipContentProps<ValueType, NameType>
) => {
  const { payload } = props;
  return (
    <Flex direction="column" gap="xs" className={styles.tooltipContent}>
      {payload?.map((entry, index) => (
        <div key={`item-${index}`} className={styles.tooltipItem}>
          <div
            className={styles.chartIcon}
            style={{ backgroundColor: entry.color }}
          />
          <span></span>
          <span className={`${styles.chartText} ${styles.chartLabel}`}>
            {entry.name}
          </span>
          <span className={styles.tooltipSpacer}></span>
          <span></span>
          <span className={`${styles.chartText} ${styles.chartValue}`}>
            {entry.value}
          </span>
        </div>
      ))}
    </Flex>
  );
};

export const renderLegend = (props: LegendContentProps) => {
  const { payload } = props;
  if (!payload || payload.length === 0) {
    return null;
  }

  return (
    <Flex direction="row" gap="md" className={styles.legendContent}>
      {payload.map((entry, index) => (
        <div key={`legend-${index}`} className={styles.legendItem}>
          <div
            className={styles.chartIcon}
            style={{ backgroundColor: entry.color }}
          />
          <span className={`${styles.chartText} ${styles.chartLabel}`}>
            {entry.value}
          </span>
        </div>
      ))}
    </Flex>
  );
};

export const renderXAxisLabel = (xAxisTitle?: string): React.ReactNode => {
  if (!xAxisTitle) return null;
  return <Label value={xAxisTitle} offset={-20} position="insideBottom" />;
};

export const renderYAxisLabel = (yAxisTitle?: string): React.ReactNode => {
  if (!yAxisTitle) return null;
  return (
    <Label value={yAxisTitle} position="insideLeft" angle={-90} offset={-20} />
  );
};

export const getPrimaryColorShades = (count: number): string[] => {
  const shades = [
    "--color-primary-200",
    "--color-primary-400",
    "--color-primary-600",
    "--color-primary-800",
  ];

  const colors: string[] = [];
  for (let i = 0; i < count; i++) {
    const shadeIndex = i % shades.length;
    colors.push(getCSSVariable(shades[shadeIndex]));
  }

  return colors;
};

export const createTooltipProps = (
  cursor?: TooltipProps<ValueType, NameType>["cursor"]
): Partial<TooltipProps<ValueType, NameType>> => ({
  content: renderToolTipContent,
  contentStyle: TOOLTIP_STYLE,
  cursor,
  filterNull: false,
});

export const createLegendProps = (
  iconType: LegendProps["iconType"] = "rect",
  wrapperStyle?: React.CSSProperties
): Partial<LegendProps> => ({
  ...COMMON_LEGEND_PROPS,
  iconType,
  wrapperStyle: wrapperStyle,
  content: renderLegend,
});
