import { create } from "zustand";
import { Layout } from "react-grid-layout";

export enum DashboardMode {
  VIEWING = "VIEWING",
  EDITING = "EDITING",
}

type DashboardStore = {
  mode: DashboardMode;
  layout: Layout[];
  saveRequested: boolean;
  setMode: (mode: DashboardMode) => void;
  setLayout: (layout: Layout[]) => void;
  requestSave: () => void;
  clearSaveRequest: () => void;
};

export const useDashboardStore = create<DashboardStore>((set) => ({
  mode: DashboardMode.VIEWING,
  layout: [],
  saveRequested: false,
  setMode: (mode) => set({ mode }),
  setLayout: (layout) => set({ layout }),
  requestSave: () => set({ saveRequested: true }),
  clearSaveRequest: () => set({ saveRequested: false }),
}));
