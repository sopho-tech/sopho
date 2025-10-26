import { create } from "zustand";
import { NotebookPageStateEnum } from "src/components/Notebooks/dto";

interface NotebookStore {
  currentNotebookId: string;
  activeCellId: string;
  notebookPageState: NotebookPageStateEnum;
  setCurrentNotebookId: (notebookId: string) => void;
  setActiveCellId: (cellId: string) => void;
  setNotebookPageState: (pageState: NotebookPageStateEnum) => void;
}

export const useNotebookStore = create<NotebookStore>((set) => ({
  currentNotebookId: "",
  activeCellId: "",
  notebookPageState: NotebookPageStateEnum.LIST,
  setCurrentNotebookId: (notebookId) => set({ currentNotebookId: notebookId }),
  setActiveCellId: (cellId) => set({ activeCellId: cellId }),
  setNotebookPageState: (pageState) =>
    set(() => ({ notebookPageState: pageState })),
}));
