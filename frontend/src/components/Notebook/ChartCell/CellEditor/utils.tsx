import { ChartType } from "src/components/Chart";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import type {
  BarChartContent,
  LineChartContent,
  PieChartContent,
  ChartContent,
} from "../../Cell/dto";

export const InfoTooltip = ({ message }: { message: string }) => (
  <div className={CellEditorStyle.infoTooltipMessage}>{message}</div>
);

export const SORT_ORDER_OPTIONS = [
  { value: "NONE", label: "None" },
  { value: "ASC", label: "Ascending" },
  { value: "DESC", label: "Descending" },
];

export const ORIENTATION_OPTIONS = [
  { value: "VERTICAL", label: "Vertical" },
  { value: "HORIZONTAL", label: "Horizontal" },
];

export const VISIBILITY_OPTIONS = [
  { value: "SHOW", label: "Show" },
  { value: "HIDE", label: "Hide" },
];

function getBarLineDefaults(chartType: ChartType, c: Record<string, unknown> | null) {
  return {
    cell_id: c?.cell_id,
    chart_type: c?.chart_type ?? chartType,
    x_axis: c?.x_axis ?? "",
    x_axis_title: c?.x_axis_title ?? "",
    y_axis: c?.y_axis ?? "",
    y_axis_aggregate_function: c?.y_axis_aggregate_function,
    y_axis_sort_order: c?.y_axis_sort_order ?? "NONE",
    y_axis_title: c?.y_axis_title ?? "",
    orientation: c?.orientation ?? "VERTICAL",
    x_axis_tick_show: c?.x_axis_tick_show ?? "SHOW",
    y_axis_tick_show: c?.y_axis_tick_show ?? "SHOW",
    axis_minor_tick_show: c?.axis_minor_tick_show ?? "SHOW",
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

export function getDefaultValuesForChart(
  chartType: ChartType | null,
  chartContent: ChartContent | null
): Record<string, unknown> {
  const c = chartContent as Record<string, unknown> | null;

  if (chartType === ChartType.BAR || chartType === ChartType.LINE) {
    return getBarLineDefaults(chartType, c);
  }

  if (chartType === ChartType.PIE) {
    return getPieDefaults(c);
  }

  if (c) {
    return { cell_id: c.cell_id, chart_type: c.chart_type };
  }

  return {};
}

export function extractBarChartFormData(formData: FormData): BarChartContent {
  return {
    x_axis: (formData.get("x_axis") as string) || "",
    y_axis: (formData.get("y_axis") as string) || "",
    chart_type: (formData.get("chart_type") as string) || undefined,
    cell_id: (formData.get("cell_id") as string) || undefined,
    orientation: (formData.get("orientation") as string) || undefined,
    y_axis_aggregate_function:
      (formData.get("y_axis_aggregate_function") as string) || undefined,
    y_axis_sort_order:
      (formData.get("y_axis_sort_order") as string) || undefined,
    x_axis_tick_show: (formData.get("x_axis_tick_show") as string) || undefined,
    y_axis_tick_show: (formData.get("y_axis_tick_show") as string) || undefined,
    axis_minor_tick_show:
      (formData.get("axis_minor_tick_show") as string) || undefined,
    x_axis_title: (formData.get("x_axis_title") as string) || undefined,
    y_axis_title: (formData.get("y_axis_title") as string) || undefined,
  };
}

export function extractLineChartFormData(formData: FormData): LineChartContent {
  return {
    x_axis: (formData.get("x_axis") as string) || "",
    y_axis: (formData.get("y_axis") as string) || "",
    chart_type: (formData.get("chart_type") as string) || undefined,
    cell_id: (formData.get("cell_id") as string) || undefined,
    orientation: (formData.get("orientation") as string) || undefined,
    y_axis_aggregate_function:
      (formData.get("y_axis_aggregate_function") as string) || undefined,
    y_axis_sort_order:
      (formData.get("y_axis_sort_order") as string) || undefined,
    x_axis_tick_show: (formData.get("x_axis_tick_show") as string) || undefined,
    y_axis_tick_show: (formData.get("y_axis_tick_show") as string) || undefined,
    axis_minor_tick_show:
      (formData.get("axis_minor_tick_show") as string) || undefined,
    show_dots: (formData.get("show_dots") as string) || "SHOW",
    x_axis_title: (formData.get("x_axis_title") as string)?.trim() || undefined,
    y_axis_title: (formData.get("y_axis_title") as string)?.trim() || undefined,
  };
}

export function extractPieChartFormData(formData: FormData): PieChartContent {
  return {
    chart_type: formData.get("chart_type") as string,
    cell_id: formData.get("cell_id") as string,
    category: formData.get("category") as string,
    value: formData.get("value") as string,
    aggregate_function: formData.get("aggregate_function") as string,
  };
}

export function extractChartFormData(
  chartType: ChartType,
  formData: FormData
): BarChartContent | LineChartContent | PieChartContent {
  switch (chartType) {
    case ChartType.BAR:
      return extractBarChartFormData(formData);
    case ChartType.LINE:
      return extractLineChartFormData(formData);
    case ChartType.PIE:
      return extractPieChartFormData(formData);
    default:
      throw Error("Chart type is not supported");
  }
}
