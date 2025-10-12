import { create } from "zustand";
import { NotebookPageStateEnum } from "src/components/Notebooks/dto";

interface NotebookStore {
  currentNotebookId: string;
  currentCellId: string;
  notebookPageState: NotebookPageStateEnum;
  setCurrentNotebookId: (notebookId: string) => void;
  setCurrentCellId: (cellId: string) => void;
  setNotebookPageState: (pageState: NotebookPageStateEnum) => void;
}

export const useNotebookStore = create<NotebookStore>((set) => ({
  currentNotebookId: "",
  currentCellId: "",
  notebookPageState: NotebookPageStateEnum.LIST,
  setCurrentNotebookId: (notebookId) => set({ currentNotebookId: notebookId }),
  setCurrentCellId: (cellId) => set({ currentCellId: cellId }),
  setNotebookPageState: (pageState) =>
    set(() => ({ notebookPageState: pageState })),
}));
