export type CanvasDto = {
  id: string | null;
  name: string | null;
  description: string | null;
  status: string | null;
  created_at: string | null;
  updated_at: string | null;
  sql_cell_count: number;
  chart_cell_count: number;
  dashboard_charts_count: number;
};

export enum CanvasesPageState {
  LIST = "LIST",
  CREATE_CANVAS_DIALOG = "CREATE_CANVAS_DIALOG",
}

export enum AggregateFunction {
  MAX = "MAX",
  MIN = "MIN",
  SUM = "SUM",
  COUNT = "COUNT",
  AVG = "AVG",
}
