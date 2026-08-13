import { ChartType, SERIES_COLOR_SLOTS } from "src/components/Chart";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import type {
  BarChartContent,
  LineChartContent,
  PieChartContent,
  MetricChartContent,
  ChartContent,
  ChartSeriesContent,
} from "src/components/Notebook/Cell/dto";

export const SERIES_LIMIT = 6;

export const BAR_LAYOUT_OPTIONS = [
  { value: "GROUPED", label: "Grouped" },
  { value: "STACKED", label: "Stacked" },
];

export function seriesAlias(
  column: string,
  aggregate: string | undefined,
): string {
  return aggregate ? `${column}_${aggregate.toLowerCase()}` : column;
}

export function nextColorIndex(series: { color_index?: number }[]): number {
  const used = new Set(
    series
      .map((entry) => entry.color_index)
      .filter((index): index is number => typeof index === "number"),
  );
  let index = 0;
  while (used.has(index)) {
    index += 1;
  }
  return index % SERIES_COLOR_SLOTS;
}

type SeriesDraft = Omit<ChartSeriesContent, "alias">;

export function withChartAliases(
  xAxis: string,
  series: SeriesDraft[],
): { xAxisAlias: string; series: ChartSeriesContent[] } {
  const seen = new Set<string>();
  const aliased: ChartSeriesContent[] = [];
  for (const entry of series) {
    const alias = seriesAlias(entry.column, entry.aggregate_function);
    if (seen.has(alias)) {
      continue;
    }
    seen.add(alias);
    aliased.push({ ...entry, alias });
  }

  let xAxisAlias = xAxis;
  let suffix = 2;
  while (seen.has(xAxisAlias)) {
    xAxisAlias = `${xAxis}_${suffix}`;
    suffix += 1;
  }

  return { xAxisAlias, series: aliased };
}

export const InfoTooltip = ({ message }: { message: string }) => (
  <div className={CellEditorStyle.infoTooltipMessage}>{message}</div>
);

export const SORT_ORDER_OPTIONS = [
  { value: "NONE", label: "None" },
  { value: "ASC", label: "Ascending" },
  { value: "DESC", label: "Descending" },
];

export const ORIENTATION_OPTIONS = [
  { value: "HORIZONTAL", label: "Horizontal" },
  { value: "VERTICAL", label: "Vertical" },
];

export const VISIBILITY_OPTIONS = [
  { value: "SHOW", label: "Show" },
  { value: "HIDE", label: "Hide" },
];

export const METRIC_FORMAT_OPTIONS = [
  { value: "DEFAULT", label: "Default" },
  { value: "PERCENTAGE", label: "Percentage" },
  { value: "CURRENCY", label: "Currency" },
];

function getBarLineDefaults(
  chartType: ChartType,
  c: Record<string, unknown> | null,
) {
  const stored = (c?.series as ChartSeriesContent[] | undefined) ?? [];
  const series = stored.length
    ? stored.map((entry, index) => ({
        column: entry.column ?? "",
        aggregate_function: entry.aggregate_function ?? "",
        label: entry.label ?? "",
        color_index: entry.color_index ?? index % SERIES_COLOR_SLOTS,
      }))
    : [{ column: "", aggregate_function: "", label: "", color_index: 0 }];

  return {
    cell_id: c?.cell_id,
    chart_type: c?.chart_type ?? chartType,
    x_axis: c?.x_axis ?? "",
    x_axis_title: c?.x_axis_title ?? "",
    series,
    y_axis_sort_by: c?.y_axis_sort_by ?? series[0].column,
    y_axis_sort_order: c?.y_axis_sort_order ?? "NONE",
    y_axis_title: c?.y_axis_title ?? "",
    orientation: c?.orientation ?? "HORIZONTAL",
    x_axis_tick_show: c?.x_axis_tick_show ?? "SHOW",
    y_axis_tick_show: c?.y_axis_tick_show ?? "SHOW",
    axis_minor_tick_show: c?.axis_minor_tick_show ?? "SHOW",
    ...(chartType === ChartType.BAR && {
      bar_layout: c?.bar_layout ?? "GROUPED",
    }),
    ...(chartType === ChartType.LINE && { show_dots: c?.show_dots ?? "SHOW" }),
  };
}

function getPieDefaults(c: Record<string, unknown> | null) {
  return {
    cell_id: c?.cell_id,
    chart_type: c?.chart_type ?? ChartType.PIE,
    category: c?.category ?? "",
    value: c?.value ?? "",
    aggregate_function: c?.aggregate_function,
  };
}

function getMetricDefaults(c: Record<string, unknown> | null) {
  return {
    cell_id: c?.cell_id,
    chart_type: c?.chart_type ?? ChartType.METRIC,
    decimal_precision: c?.decimal_precision ?? 2,
    suffix: c?.suffix ?? "",
    format: c?.format ?? "DEFAULT",
  };
}

export function getDefaultValuesForChart(
  chartType: ChartType | null,
  chartContent: ChartContent | null,
): Record<string, unknown> {
  const c = chartContent as Record<string, unknown> | null;

  if (chartType === ChartType.BAR || chartType === ChartType.LINE) {
    return getBarLineDefaults(chartType, c);
  }

  if (chartType === ChartType.PIE) {
    return getPieDefaults(c);
  }

  if (chartType === ChartType.METRIC) {
    return getMetricDefaults(c);
  }

  if (c) {
    return { cell_id: c.cell_id, chart_type: c.chart_type };
  }

  return {};
}

export function extractAxisChartData(
  chartType: ChartType,
  values: Record<string, unknown>,
): BarChartContent | LineChartContent {
  const xAxis = (values.x_axis as string) || "";
  const rawSeries = (values.series as ChartSeriesContent[] | undefined) ?? [];
  const { xAxisAlias, series } = withChartAliases(
    xAxis,
    rawSeries
      .filter((entry) => Boolean(entry?.column))
      .slice(0, SERIES_LIMIT)
      .map((entry, index) => ({
        column: entry.column,
        aggregate_function: entry.aggregate_function || undefined,
        label: (entry.label as string)?.trim() || null,
        color_index: entry.color_index ?? index % SERIES_COLOR_SLOTS,
      })),
  );

  const sortByColumn = values.y_axis_sort_by as string | undefined;
  const sortBySeries =
    series.find((entry) => entry.column === sortByColumn) ?? series[0];

  return {
    x_axis: xAxis,
    x_axis_alias: xAxisAlias,
    series,
    chart_type: (values.chart_type as string) || undefined,
    cell_id: (values.cell_id as string) || undefined,
    orientation: (values.orientation as string) || undefined,
    y_axis_sort_by: sortBySeries?.alias,
    y_axis_sort_order: (values.y_axis_sort_order as string) || undefined,
    x_axis_tick_show: (values.x_axis_tick_show as string) || undefined,
    y_axis_tick_show: (values.y_axis_tick_show as string) || undefined,
    axis_minor_tick_show: (values.axis_minor_tick_show as string) || undefined,
    x_axis_title: (values.x_axis_title as string)?.trim() || undefined,
    y_axis_title: (values.y_axis_title as string)?.trim() || undefined,
    ...(chartType === ChartType.BAR && {
      bar_layout: (values.bar_layout as string) || undefined,
    }),
    ...(chartType === ChartType.LINE && {
      show_dots: (values.show_dots as string) || "SHOW",
    }),
  };
}

export function extractPieChartData(
  values: Record<string, unknown>,
): PieChartContent {
  return {
    chart_type: values.chart_type as string,
    cell_id: values.cell_id as string,
    category: values.category as string,
    value: values.value as string,
    aggregate_function: values.aggregate_function as string,
  };
}

export function extractMetricChartData(
  values: Record<string, unknown>,
): MetricChartContent {
  const decimalPrecision = values.decimal_precision;
  const suffix = values.suffix as string;
  const format = values.format as string;
  return {
    chart_type: values.chart_type as string,
    cell_id: values.cell_id as string,
    decimal_precision:
      decimalPrecision != null && decimalPrecision !== ""
        ? Number(decimalPrecision)
        : undefined,
    suffix: suffix?.trim() || undefined,
    format:
      format === "PERCENTAGE" || format === "CURRENCY" || format === "DEFAULT"
        ? format
        : undefined,
  };
}

export function extractChartData(
  chartType: ChartType,
  values: Record<string, unknown>,
): BarChartContent | LineChartContent | PieChartContent | MetricChartContent {
  switch (chartType) {
    case ChartType.BAR:
    case ChartType.LINE:
      return extractAxisChartData(chartType, values);
    case ChartType.PIE:
      return extractPieChartData(values);
    case ChartType.METRIC:
      return extractMetricChartData(values);
    default:
      throw Error("Chart type is not supported");
  }
}
