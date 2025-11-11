import { create } from "zustand";
import type {
  CellOutputState,
  ExecuteCellResponseDto,
  ExecutionState,
} from "src/components/Notebooks/Notebook/Cell/dto";

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

type ChartCellExecutionStore = {
  outputs: Record<string, ExecuteCellResponseDto>;
  executionStates: Record<string, ExecutionState>;
  outputStates: Record<string, CellOutputState>;
  setExecutionState: (cellId: string, executionState: ExecutionState) => void;
  getExecutionState: (cellId: string) => ExecutionState | undefined;
  setOutputState: (cellId: string, outputState: CellOutputState) => void;
  getOutputState: (cellId: string) => CellOutputState | undefined;
  setOutput: (cellId: string, output: ExecuteCellResponseDto) => void;
  clearOutput: (cellId: string) => void;
  clearAll: () => void;
  getOutput: (cellId: string) => ExecuteCellResponseDto | undefined;
};

export const useChartCellExecutionStore = create<ChartCellExecutionStore>(
  (set, get) => ({
    outputs: {},
    executionStates: {},
    outputStates: {},
    setExecutionState: (cellId, executionState) =>
      set((state) => ({
        executionStates: { ...state.executionStates, [cellId]: executionState },
      })),
    getExecutionState: (cellId) => get().executionStates[cellId],
    setOutputState: (cellId, outputState) =>
      set((state) => ({
        outputStates: { ...state.outputStates, [cellId]: outputState },
      })),
    getOutputState: (cellId) => get().outputStates[cellId],
    getOutput: (cellId) => get().outputs[cellId],
    setOutput: (cellId, output) =>
      set((state) => ({ outputs: { ...state.outputs, [cellId]: output } })),
    clearOutput: (cellId) =>
      set((state) => {
        const { [cellId]: _, ...rest } = state.outputs;
        return { outputs: rest };
      }),
    clearAll: () => set({ outputs: {} }),
  })
);
