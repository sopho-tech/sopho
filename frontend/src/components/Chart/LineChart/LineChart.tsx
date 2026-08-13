import { useMemo } from "react";
import { Line, LineChart as RechartsLineChart } from "recharts";
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

export type LineChartType = "linear" | "monotone" | "step";

export type LineChartProps = {
  type?: LineChartType;
  xAxis: string;
  series: ChartSeriesSpec[];
  data: object[];
  xAxisTitle?: string;
  yAxisTitle?: string;
  showDots?: boolean;
  xAxisTickShow?: string;
  yAxisTickShow?: string;
  orientation?: ChartOrientation;
};

export function LineChart({
  type = "monotone",
  xAxis,
  series,
  data,
  xAxisTitle,
  yAxisTitle,
  showDots = true,
  xAxisTickShow,
  yAxisTickShow,
  orientation,
}: LineChartProps) {
  const showXTicks = xAxisTickShow !== "HIDE";
  const showYTicks = yAxisTickShow !== "HIDE";
  const isVertical = orientation === "VERTICAL";
  const activeDot = useMemo(() => ({ r: 7 }), []);

  return (
    <ChartContainer>
      <RechartsLineChart
        responsive
        data={data}
        margin={getChartMargins(orientation)}
        layout={getRechartsLayout(orientation)}
      >
        <ChartCartesianGrid />
        {series.map((entry) => (
          <Line
            key={entry.dataKey}
            type={type}
            dataKey={entry.dataKey}
            name={entry.name}
            stroke={getSeriesColor(entry.colorIndex)}
            strokeWidth={1}
            activeDot={activeDot}
            dot={showDots}
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
      </RechartsLineChart>
    </ChartContainer>
  );
}
