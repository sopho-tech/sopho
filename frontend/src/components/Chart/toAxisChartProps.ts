import type {
  ChartOrientation,
  ChartSeriesSpec,
} from "src/components/Chart/ChartCommon";
import type {
  BarChartContent,
  LineChartContent,
} from "src/components/Notebook/Cell/dto";

export function toChartSeriesSpecs(
  content: BarChartContent | LineChartContent
): ChartSeriesSpec[] {
  return (content.series ?? []).map((entry, index) => ({
    dataKey: entry.alias,
    name: entry.label || entry.column,
    colorIndex: entry.color_index ?? index,
  }));
}

export function findMissingSeriesColumns(
  series: ChartSeriesSpec[],
  resultColumns: string[]
): string[] {
  const available = new Set(resultColumns);
  return series
    .filter((entry) => !available.has(entry.dataKey))
    .map((entry) => entry.name);
}

export function toAxisChartProps(content: BarChartContent | LineChartContent) {
  return {
    xAxis: content.x_axis_alias,
    series: toChartSeriesSpecs(content),
    xAxisTitle: content.x_axis_title,
    yAxisTitle: content.y_axis_title,
    xAxisTickShow: content.x_axis_tick_show,
    yAxisTickShow: content.y_axis_tick_show,
    orientation: content.orientation as ChartOrientation | undefined,
  };
}
