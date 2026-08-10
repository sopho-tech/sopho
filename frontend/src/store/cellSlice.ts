import { StateCreator } from "zustand";
import type { ChartContent } from "src/components/Notebook/Cell/dto";
import { ExecutionState } from "src/components/Notebook/Cell/dto";

export type CellSlice = {
  cell: {
    chartContents: Record<string, ChartContent | null>;
    setChartContent: (cellId: string, chartContent: ChartContent | null) => void;
    executionStates: Record<string, ExecutionState>;
    startedAt: Record<string, number>;
    setExecutionState: (cellId: string, executionState: ExecutionState) => void;
    clearExecutionState: (cellId: string) => void;
  };
};

export const createCellSlice: StateCreator<CellSlice> = (set) => ({
  cell: {
    chartContents: {},
    executionStates: {},
    startedAt: {},
    setChartContent: (cellId, newChartContent) =>
      set((state) => ({
        cell: {
          ...state.cell,
          chartContents: { ...state.cell.chartContents, [cellId]: newChartContent },
        },
      })),
    setExecutionState: (cellId, executionState) =>
      set((state) => {
        const startedAt = { ...state.cell.startedAt };
        if (executionState === ExecutionState.RUNNING) {
          startedAt[cellId] = Date.now();
        } else {
          delete startedAt[cellId];
        }
        return {
          cell: {
            ...state.cell,
            executionStates: {
              ...state.cell.executionStates,
              [cellId]: executionState,
            },
            startedAt,
          },
        };
      }),
    clearExecutionState: (cellId) =>
      set((state) => {
        const executionStates = { ...state.cell.executionStates };
        const startedAt = { ...state.cell.startedAt };
        delete executionStates[cellId];
        delete startedAt[cellId];
        return { cell: { ...state.cell, executionStates, startedAt } };
      }),
  },
});
