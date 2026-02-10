import { StateCreator } from "zustand";

export type NotebookSlice = {
  notebook: {
    activeCellId: string;
    setActiveCellId: (cellId: string) => void;
  };
};

export const createNotebookSlice: StateCreator<NotebookSlice> = (set) => ({
  notebook: {
    activeCellId: "",
    setActiveCellId: (cellId) =>
      set((state) => ({ notebook: { ...state.notebook, activeCellId: cellId } })),
  },
});
