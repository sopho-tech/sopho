import { Chart } from "src/components/Chart";
import type { EChartsOption } from "echarts";

export type BarChartProps = {
  xAxis: string;
  yAxis: string;
  dimensions: string[];
  data: Object[];
  orientation?: string;
  sortOrder?: string;
  axisTickShow?: string;
  axisMinorTickShow?: string;
};

export function BarChart({
  xAxis,
  yAxis,
  data,
  dimensions,
  orientation = "VERTICAL",
  sortOrder,
  axisTickShow,
  axisMinorTickShow,
}: BarChartProps) {
  const isHorizontal = orientation === "HORIZONTAL";

  const categoryAxisConfig = {
    type: "category" as const,
    axisLabel: {
      interval: 0,
      fontFamily: "monospace",
    },
    axisTick: {
      interval: 0,
      show: !axisTickShow || axisTickShow === "SHOW",
      alignWithLabel: true,
    },
    minorTick: {
      show: !axisMinorTickShow || axisMinorTickShow === "SHOW",
    },
  };

  const shouldSort = sortOrder && sortOrder !== "NONE";
  const sortOrderValue = sortOrder === "DESC" ? "desc" : "asc";

  const dataset: EChartsOption["dataset"] = [
    {
      dimensions: dimensions,
      source: data,
    },
  ];

  if (shouldSort) {
    dataset.push({
      transform: [
        {
          type: "sort",
          config: { dimension: yAxis, order: sortOrderValue },
        },
      ],
    });
  }

  const option: EChartsOption = {
    legend: {
      orient: "vertical",
      right: 10,
      top: 20,
      bottom: 20,
    },
    tooltip: {},
    dataset: dataset,
    xAxis: isHorizontal ? {} : categoryAxisConfig,
    yAxis: isHorizontal ? categoryAxisConfig : {},
    series: [
      {
        type: "bar",
        name: yAxis,
        encode: isHorizontal ? { x: yAxis, y: xAxis } : { x: xAxis, y: yAxis },
        datasetIndex: shouldSort ? 1 : 0,
      },
    ],
  };
  return <Chart option={option}></Chart>;
}
