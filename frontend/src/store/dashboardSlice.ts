import { StateCreator } from "zustand";
import { Layout } from "react-grid-layout";
import { DashboardDto, convertDtoToRGLayout } from "src/components/Dashboard/dto";

export enum DashboardMode {
  VIEWING = "VIEWING",
  EDITING = "EDITING",
}

export type DashboardSlice = {
  dashboard: {
    mode: DashboardMode;
    layout: Layout[];
    saveRequested: boolean;
    showChartBrowser: boolean;
    setShowChartBrowser: (showChartBrowser: boolean) => void;
    setMode: (mode: DashboardMode) => void;
    setLayout: (layout: Layout[]) => void;
    getLayout: () => Layout[];
    requestSave: () => void;
    clearSaveRequest: () => void;
    resetFromBackendData: (dashboardData: DashboardDto) => void;
  };
};

export const createDashboardSlice: StateCreator<DashboardSlice> = (set, get) => ({
  dashboard: {
    mode: DashboardMode.VIEWING,
    layout: [],
    saveRequested: false,
    showChartBrowser: false,
    setShowChartBrowser: (showChartBrowser: boolean) =>
      set((state) => ({ dashboard: { ...state.dashboard, showChartBrowser } })),
    setMode: (mode) =>
      set((state) => ({ dashboard: { ...state.dashboard, mode } })),
    setLayout: (layout) =>
      set((state) => ({ dashboard: { ...state.dashboard, layout } })),
    getLayout: () => get().dashboard.layout,
    requestSave: () =>
      set((state) => ({ dashboard: { ...state.dashboard, saveRequested: true } })),
    clearSaveRequest: () =>
      set((state) => ({ dashboard: { ...state.dashboard, saveRequested: false } })),
    resetFromBackendData: (dashboardData) => {
      if (!dashboardData?.layout) {
        return;
      }
      const layout = convertDtoToRGLayout(dashboardData.layout);
      set((state) => ({ dashboard: { ...state.dashboard, layout } }));
    },
  },
});
