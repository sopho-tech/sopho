import { create } from "zustand";
import { Layout } from "react-grid-layout";
import { DashboardDto } from "src/components/Dashboard/dto";
import { convertDtoToRGLayout } from "src/components/Dashboard/dto";

export enum DashboardMode {
  VIEWING = "VIEWING",
  EDITING = "EDITING",
}

type DashboardStore = {
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

export const useDashboardStore = create<DashboardStore>((set, get) => ({
  mode: DashboardMode.VIEWING,
  layout: [],
  saveRequested: false,
  showChartBrowser: false,
  setShowChartBrowser: (showChartBrowser: boolean) => set({ showChartBrowser }),
  setMode: (mode) => set({ mode }),
  setLayout: (layout) => set({ layout }),
  getLayout: () => get().layout,
  requestSave: () => set({ saveRequested: true }),
  clearSaveRequest: () => set({ saveRequested: false }),
  resetFromBackendData: (dashboardData) => {
    if (!dashboardData?.layout) {
      return;
    }
    const layout = convertDtoToRGLayout(dashboardData.layout);
    set({ layout });
  },
}));
