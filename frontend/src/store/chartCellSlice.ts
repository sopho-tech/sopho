import { StateCreator } from "zustand";
import type {
  CellOutputState,
  ExecuteCellResponseDto,
  ExecutionState,
} from "src/components/Notebook/Cell/dto";

export type ChartCellSlice = {
  chartCell: {
    forms: Record<string, unknown>;
    outputs: Record<string, ExecuteCellResponseDto>;
    executionStates: Record<string, ExecutionState>;
    outputStates: Record<string, CellOutputState>;
    setForm: (cellId: string, form: unknown) => void;
    getForm: (cellId: string) => unknown;
    setExecutionState: (cellId: string, executionState: ExecutionState) => void;
    getExecutionState: (cellId: string) => ExecutionState | undefined;
    setOutputState: (cellId: string, outputState: CellOutputState) => void;
    getOutputState: (cellId: string) => CellOutputState | undefined;
    setOutput: (cellId: string, output: ExecuteCellResponseDto) => void;
    clearOutput: (cellId: string) => void;
    clearAll: () => void;
    getOutput: (cellId: string) => ExecuteCellResponseDto | undefined;
  };
};

export const createChartCellSlice: StateCreator<ChartCellSlice> = (set, get) => ({
  chartCell: {
    forms: {},
    outputs: {},
    executionStates: {},
    outputStates: {},
    setForm: (cellId: string, form: unknown) =>
      set((state) => ({
        chartCell: {
          ...state.chartCell,
          forms: { ...state.chartCell.forms, [cellId]: form },
        },
      })),
    getForm: (cellId: string) => get().chartCell.forms[cellId],
    setExecutionState: (cellId, executionState) =>
      set((state) => ({
        chartCell: {
          ...state.chartCell,
          executionStates: {
            ...state.chartCell.executionStates,
            [cellId]: executionState,
          },
        },
      })),
    getExecutionState: (cellId) => get().chartCell.executionStates[cellId],
    setOutputState: (cellId, outputState) =>
      set((state) => ({
        chartCell: {
          ...state.chartCell,
          outputStates: { ...state.chartCell.outputStates, [cellId]: outputState },
        },
      })),
    getOutputState: (cellId) => get().chartCell.outputStates[cellId],
    getOutput: (cellId) => get().chartCell.outputs[cellId],
    setOutput: (cellId, output) =>
      set((state) => ({
        chartCell: {
          ...state.chartCell,
          outputs: { ...state.chartCell.outputs, [cellId]: output },
        },
      })),
    clearOutput: (cellId) =>
      set((state) => {
        const rest = Object.fromEntries(
          Object.entries(state.chartCell.outputs).filter(([k]) => k !== cellId)
        );
        return { chartCell: { ...state.chartCell, outputs: rest } };
      }),
    clearAll: () =>
      set((state) => ({ chartCell: { ...state.chartCell, outputs: {} } })),
  },
});
