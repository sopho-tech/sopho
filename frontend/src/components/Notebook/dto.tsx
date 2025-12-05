import { CellDto } from "src/components/Notebook/Cell/dto";

export type NotebookDto = {
  id: string | null;
  name: string | null;
  description: string | null;
  status: string | null;
  cells: CellDto[] | null;
  created_at: string | null;
  updated_at: string | null;
};

export enum AggregateFunction {
  MAX = "MAX",
  MIN = "MIN",
  SUM = "SUM",
  COUNT = "COUNT",
  AVG = "AVG",
}
