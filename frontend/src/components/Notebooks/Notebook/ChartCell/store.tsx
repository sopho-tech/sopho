import { create } from "zustand";

interface ChartCellStore {
  forms: Record<string, any>;
  setForm: (cellId: string, form: any) => void;
  getForm: (cellId: string) => any;
}

export const useChartCellOutputStore = create<ChartCellStore>((set, get) => ({
  forms: {},
  setForm: (cellId: string, form: any) =>
    set((state) => ({ forms: { ...state.forms, [cellId]: form } })),
  getForm: (cellId: string) => get().forms[cellId],
}));
