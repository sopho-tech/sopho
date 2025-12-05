import { create } from "zustand";

interface NotebookStore {
  activeCellId: string;
  setActiveCellId: (cellId: string) => void;
}

export const useNotebookStore = create<NotebookStore>((set) => ({
  activeCellId: "",
  setActiveCellId: (cellId) => set({ activeCellId: cellId }),
}));
