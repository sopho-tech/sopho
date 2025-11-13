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

export type ChartContent = {
  x_axis: string;
  y_axis: string;
  chart_type?: string;
  cell_id?: string;
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
  column_names: string[] | null;
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
