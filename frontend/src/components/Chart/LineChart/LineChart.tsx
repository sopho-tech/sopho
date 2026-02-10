import { Line, LineChart as RechartsLineChart } from "recharts";
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

export type LineChartType = "linear" | "monotone" | "step";

export type LineChartProps = {
  type?: LineChartType;
  xAxis: string;
  yAxis: string;
  data: Object[];
  xAxisTitle?: string;
  yAxisTitle?: string;
  showDots?: boolean;
  xAxisTickShow?: string;
  yAxisTickShow?: string;
};

export function LineChart({
  type = "monotone",
  xAxis,
  yAxis,
  data,
  xAxisTitle,
  yAxisTitle,
  showDots = true,
  xAxisTickShow,
  yAxisTickShow,
}: LineChartProps) {
  const showXTicks = xAxisTickShow !== "HIDE";
  const showYTicks = yAxisTickShow !== "HIDE";
  return (
    <ChartContainer>
      <RechartsLineChart responsive data={data} margin={{ ...CHART_MARGINS }}>
        <ChartCartesianGrid />
        <Line
          type={type}
          dataKey={yAxis}
          stroke={getCSSVariable("--color-primary-500")}
          strokeWidth={1}
          name={yAxis}
          activeDot={{ r: 7 }}
          dot={showDots}
        />
        <ChartXAxis dataKey={xAxis} label={xAxisTitle} showTicks={showXTicks} />
        <ChartYAxis label={yAxisTitle} showTicks={showYTicks} />
        <ChartLegend position="top" />
        <ChartTooltip />
      </RechartsLineChart>
    </ChartContainer>
  );
}
