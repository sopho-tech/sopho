import { StateCreator } from "zustand";
import type { ChartContent } from "src/components/Notebook/Cell/dto";

export type CellSlice = {
  cell: {
    chartContents: Record<string, ChartContent | null>;
    setChartContent: (cellId: string, chartContent: ChartContent | null) => void;
  };
};

export const createCellSlice: StateCreator<CellSlice> = (set) => ({
  cell: {
    chartContents: {},
    setChartContent: (cellId, newChartContent) =>
      set((state) => ({
        cell: {
          ...state.cell,
          chartContents: { ...state.cell.chartContents, [cellId]: newChartContent },
        },
      })),
  },
});
