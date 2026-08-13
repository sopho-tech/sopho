import { Bar, BarChart as RechartsBarChart } from "recharts";
import {
  ChartCartesianGrid,
  ChartContainer,
  ChartLegend,
  ChartTooltip,
  ChartXAxis,
  ChartYAxis,
  getChartMargins,
  getRechartsLayout,
  getSeriesColor,
  type ChartOrientation,
  type ChartSeriesSpec,
} from "src/components/Chart/ChartCommon";

const SERIES_STACK_ID = "series";

export type BarChartProps = {
  xAxis: string;
  series: ChartSeriesSpec[];
  data: object[];
  xAxisTitle?: string;
  yAxisTitle?: string;
  showDots?: boolean;
  xAxisTickShow?: string;
  yAxisTickShow?: string;
  orientation?: ChartOrientation;
  stacked?: boolean;
};

export function BarChart({
  xAxis,
  series,
  data,
  xAxisTitle,
  yAxisTitle,
  xAxisTickShow,
  yAxisTickShow,
  orientation,
  stacked = false,
}: BarChartProps) {
  const showXTicks = xAxisTickShow !== "HIDE";
  const showYTicks = yAxisTickShow !== "HIDE";
  const isVertical = orientation === "VERTICAL";

  return (
    <ChartContainer>
      <RechartsBarChart
        responsive
        data={data}
        margin={getChartMargins(orientation)}
        layout={getRechartsLayout(orientation)}
      >
        <ChartCartesianGrid />
        {series.map((entry) => (
          <Bar
            key={entry.dataKey}
            dataKey={entry.dataKey}
            name={entry.name}
            fill={getSeriesColor(entry.colorIndex)}
            stackId={stacked ? SERIES_STACK_ID : undefined}
          />
        ))}
        <ChartXAxis
          dataKey={isVertical ? undefined : xAxis}
          type={isVertical ? "number" : "category"}
          label={xAxisTitle}
          showTicks={showXTicks}
        />
        <ChartYAxis
          dataKey={isVertical ? xAxis : undefined}
          type={isVertical ? "category" : "number"}
          label={yAxisTitle}
          showTicks={showYTicks}
        />
        <ChartLegend position="top" />
        <ChartTooltip />
      </RechartsBarChart>
    </ChartContainer>
  );
}
