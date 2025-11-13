import { Chart } from "src/components/Chart";
import type { EChartsOption } from "echarts";

export type BarChartProps = {
  xAxis: string;
  yAxis: string;
  data: Object[];
};

export function BarChart({ xAxis, yAxis, data }: BarChartProps) {
  const option: EChartsOption = {
    legend: {
      orient: "vertical",
      right: 10,
      top: 20,
      bottom: 20,
    },
    tooltip: {},
    dataset: {
      source: data,
    },
    xAxis: { type: "category" },
    yAxis: {},
    series: [{ type: "bar", name: yAxis, encode: { x: xAxis, y: yAxis } }],
  };
  return <Chart option={option}></Chart>;
}
