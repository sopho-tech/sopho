import { Bar, BarChart as RechartsBarChart } from "recharts";
import { getCSSVariable } from "src/utils/css_util";
import {
  CHART_MARGINS,
  ChartCartesianGrid,
  ChartContainer,
  ChartLegend,
  ChartTooltip,
  ChartXAxis,
  ChartYAxis,
} from "../ChartCommon";

export type BarChartProps = {
  xAxis: string;
  yAxis: string;
  data: Object[];
  xAxisTitle?: string;
  yAxisTitle?: string;
  showDots?: boolean;
  xAxisTickShow?: string;
  yAxisTickShow?: string;
};

export function BarChart({
  xAxis,
  yAxis,
  data,
  xAxisTitle,
  yAxisTitle,
  xAxisTickShow,
  yAxisTickShow,
}: BarChartProps) {
  const showXTicks = xAxisTickShow !== "HIDE";
  const showYTicks = yAxisTickShow !== "HIDE";
  return (
    <ChartContainer aspect={1.618}>
      <RechartsBarChart responsive data={data} margin={CHART_MARGINS}>
        <ChartCartesianGrid />
        <Bar
          dataKey={yAxis}
          fill={getCSSVariable("--color-primary-500")}
          name={yAxis}
        />
        <ChartXAxis dataKey={xAxis} label={xAxisTitle} showTicks={showXTicks} />
        <ChartYAxis label={yAxisTitle} showTicks={showYTicks} />
        <ChartLegend position="top" />
        <ChartTooltip />
      </RechartsBarChart>
    </ChartContainer>
  );
}
