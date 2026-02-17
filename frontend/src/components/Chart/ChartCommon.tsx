import {
  CartesianGrid,
  Label,
  Legend,
  LegendProps,
  ResponsiveContainer,
  Tooltip,
  TooltipContentProps,
  TooltipProps,
  XAxis,
  YAxis,
} from "recharts";
import {
  NameType,
  ValueType,
} from "recharts/types/component/DefaultTooltipContent";
import type { Props as LegendContentProps } from "recharts/types/component/DefaultLegendContent";
import { Flex } from "src/components/design-system";
import { getCSSVariable } from "src/utils/css_util";
import styles from "./ChartCommon.module.css";

export type ChartContainerProps = {
  children: React.ReactNode;
  aspect?: number;
};

const INITIAL_DIMENSION = { width: 1, height: 1 };

export function ChartContainer({ children, aspect }: ChartContainerProps) {
  return (
    <ResponsiveContainer
      width="100%"
      {...(aspect != null && { aspect })}
      initialDimension={INITIAL_DIMENSION}
      debounce={300}
      className={styles.container}
    >
      {children}
    </ResponsiveContainer>
  );
}

export function ChartCartesianGrid() {
  return (
    <CartesianGrid
      stroke={getCSSVariable("--color-grey-300")}
      strokeDasharray={"4 1 2"}
    />
  );
}

export type ChartXAxisProps = {
  dataKey: string;
  label?: string;
  showTicks?: boolean;
};

export function ChartXAxis({
  dataKey,
  label,
  showTicks = true,
}: ChartXAxisProps) {
  return (
    <XAxis
      dataKey={dataKey}
      stroke={getCSSVariable("--color-grey-600")}
      minTickGap={20}
      tick={showTicks}
      tickLine={showTicks}
    >
      {renderXAxisLabel(label)}
    </XAxis>
  );
}

export type ChartYAxisProps = {
  label?: string;
  showTicks?: boolean;
};

export function ChartYAxis({ label, showTicks = true }: ChartYAxisProps) {
  return (
    <YAxis
      stroke={getCSSVariable("--color-grey-600")}
      tick={showTicks}
      tickLine={showTicks}
    >
      {renderYAxisLabel(label)}
    </YAxis>
  );
}

export const CHART_MARGINS = {
  top: 10,
  right: 60,
  bottom: 50,
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
  verticalAlign: "top",
  iconSize: 16,
  inactiveColor: "#ccc",
};

export const renderToolTipContent = (
  props: TooltipContentProps<ValueType, NameType>
) => {
  const { label, payload } = props;
  return (
    <Flex direction="column" gap="xs" className={styles.tooltipContent}>
      {label && <div className={styles.tooltipHeading}>{label}</div>}
      {payload?.map((entry, index) => (
        <div key={`item-${index}`} className={styles.tooltipItem}>
          <div
            className={styles.chartIcon}
            style={
              // eslint-disable-next-line react-perf/jsx-no-new-object-as-prop
              { backgroundColor: entry.color ?? entry.payload.fill }
            }
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
  const { payload, layout } = props;
  if (!payload || payload.length === 0) {
    return null;
  }

  return (
    <Flex
      direction={layout === "vertical" ? "column" : "row"}
      gap="md"
      className={`${styles.legendContent} ${layout === "vertical" ? styles.legendContentRight : ""}`}
    >
      {payload.map((entry, index) => (
        <div key={`legend-${index}`} className={styles.legendItem}>
          <div
            className={styles.chartIcon}
            style={
              // eslint-disable-next-line react-perf/jsx-no-new-object-as-prop
              { backgroundColor: entry.color }
            }
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
    <Label value={yAxisTitle} position="insideLeft" angle={-90} offset={-10} />
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

const CHART_TOOLTIP_PROPS: Partial<TooltipProps<ValueType, NameType>> = {
  content: renderToolTipContent,
  contentStyle: TOOLTIP_STYLE,
  cursor: { fill: "rgba(0, 0, 0, 0.05)" },
  filterNull: false,
};

const CHART_LEGEND_PROPS: Partial<LegendProps> = {
  ...COMMON_LEGEND_PROPS,
  iconType: "rect",
  wrapperStyle: { paddingTop: "1rem" },
  content: renderLegend,
};

export type ChartLegendPosition = "top" | "right";

const LEGEND_POSITION_PROPS: Record<
  ChartLegendPosition,
  Pick<LegendProps, "align" | "verticalAlign" | "layout">
> = {
  top: { align: "center", verticalAlign: "top", layout: "horizontal" },
  right: { align: "right", verticalAlign: "middle", layout: "vertical" },
};

export type ChartLegendProps = {
  position: ChartLegendPosition;
};

const LEGEND_WRAPPER_STYLE = { alignContent: "start" as const };

export function ChartLegend({ position }: ChartLegendProps) {
  return (
    <Legend
      {...CHART_LEGEND_PROPS}
      {...LEGEND_POSITION_PROPS[position]}
      wrapperStyle={LEGEND_WRAPPER_STYLE}
    />
  );
}

export function ChartTooltip() {
  return <Tooltip {...CHART_TOOLTIP_PROPS} />;
}
