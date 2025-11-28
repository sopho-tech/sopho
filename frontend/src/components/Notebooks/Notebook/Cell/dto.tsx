import { ColumnDataType } from "src/constants/database_types";

export enum CellType {
  TEXT = "TEXT",
  CODE = "CODE",
  MARKDOWN = "MARKDOWN",
  SQL = "SQL",
  CHART = "CHART",
}

export enum CellOutputState {
  ABSENT = "ABSENT",
  PRESENT = "PRESENT",
  EXECUTING = "EXECUTING",
  ERROR = "ERROR",
}

export enum ExecutionState {
  PENDING = "PENDING",
  RUNNING = "RUNNING",
  COMPLETED = "COMPLETED",
  FAILED = "FAILED",
}

export enum AxisTickShow {
  SHOW = "SHOW",
  HIDE = "HIDE",
}

export enum AxisMinorTickShow {
  SHOW = "SHOW",
  HIDE = "HIDE",
}

export type ChartContent = {
  x_axis: string;
  y_axis: string;
  chart_type?: string;
  cell_id?: string;
  orientation?: string;
  y_axis_aggregate_function?: string;
  y_axis_sort_order?: string;
  axis_tick_show?: string;
  axis_minor_tick_show?: string;
};

export type CellDto = {
  id: string;
  notebook_id: string | null;
  connection_id: string | null;
  display_order: number | null;
  name: string | null;
  content: string | null;
  cell_type: CellType | null;
  status: string | null;
};

export type CreateCellDto = {
  name: string | null;
  content: string | null;
  cell_type: CellType | null;
  notebook_id: string | null;
};

export type ExecuteCellResponseDto = {
  columns: Array<{ column_name: string; data_type: ColumnDataType }> | null;
  data: Record<string, unknown>[] | null;
};

export function getChartContent(cell: CellDto): ChartContent | null {
  if (cell.cell_type !== CellType.CHART || !cell.content) {
    return null;
  }
  try {
    return JSON.parse(cell.content) as ChartContent;
  } catch {
    return null;
  }
}

export function serializeChartContent(content: ChartContent): string {
  return JSON.stringify(content);
}
