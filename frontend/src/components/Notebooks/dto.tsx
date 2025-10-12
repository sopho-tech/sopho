import { CellDto } from "src/components/Notebooks/Notebook/Cell/dto";

export type NotebookDto = {
  id: string | null;
  name: string | null;
  description: string | null;
  status: string | null;
  cells: CellDto[] | null;
  created_at: string | null;
  updated_at: string | null;
};

export enum NotebookPageStateEnum {
  LIST = "LIST",
  CREATE_NOTEBOOK_DIALOG = "CREATE_NOTEBOOK_DIALOG",
}
