import { StateCreator } from "zustand";
import type {
  CellOutputState,
  ChartContent,
  ExecuteCellResponseDto,
  ExecutionState,
} from "src/components/Notebook/Cell/dto";

export type CellSlice = {
  cell: {
    outputs: Record<string, ExecuteCellResponseDto>;
    executionStates: Record<string, ExecutionState>;
    outputStates: Record<string, CellOutputState>;
    // Content that is being edited in the frontend. It may or may not be persisted in the backend.
    chartContents: Record<string, ChartContent>;
    setExecutionState: (cellId: string, executionState: ExecutionState) => void;
    getExecutionState: (cellId: string) => ExecutionState | undefined;
    setOutputState: (cellId: string, outputState: CellOutputState) => void;
    getOutputState: (cellId: string) => CellOutputState | undefined;
    setOutput: (cellId: string, output: ExecuteCellResponseDto) => void;
    clearOutput: (cellId: string) => void;
    clearAll: () => void;
    getOutput: (cellId: string) => ExecuteCellResponseDto | undefined;
    setChartContent: (cellId:string, chartContent: ChartContent) => void;
  };
};

export const createCellSlice: StateCreator<CellSlice> = (set, get) => ({
  cell: {
    outputs: {},
    executionStates: {},
    outputStates: {},
    chartContents: {},
    setExecutionState: (cellId, executionState) =>
      set((state) => ({
        cell: {
          ...state.cell,
          executionStates: { ...state.cell.executionStates, [cellId]: executionState },
        },
      })),
    getExecutionState: (cellId) => get().cell.executionStates[cellId],
    setOutputState: (cellId, outputState) =>
      set((state) => ({
        cell: {
          ...state.cell,
          outputStates: { ...state.cell.outputStates, [cellId]: outputState },
        },
      })),
    getOutputState: (cellId) => get().cell.outputStates[cellId],
    getOutput: (cellId) => get().cell.outputs[cellId],
    setOutput: (cellId, output) =>
      set((state) => ({
        cell: {
          ...state.cell,
          outputs: { ...state.cell.outputs, [cellId]: output },
        },
      })),
    clearOutput: (cellId) =>
      set((state) => {
        const { [cellId]: _, ...rest } = state.cell.outputs;
        return { cell: { ...state.cell, outputs: rest } };
      }),
    clearAll: () =>
      set((state) => ({ cell: { ...state.cell, outputs: {} } })),
    setChartContent: (cellId, newChartContent) => set((state) => ({
      cell: {
        ...state.cell,
        chartContents: {...state.cell.chartContents, [cellId]: newChartContent},
      },
    })),
  },
});
