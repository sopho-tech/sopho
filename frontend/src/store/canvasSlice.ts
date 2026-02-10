import { StateCreator } from "zustand";
import { CanvasesPageState } from "src/components/Canvases/dto";

export type CanvasSlice = {
  canvas: {
    currentCanvasId: string;
    activeNotebookId: string;
    canvasPageState: CanvasesPageState;
    setCurrentCanvasId: (canvasId: string) => void;
    setActiveNotebookId: (notebookId: string) => void;
    setCanvasPageState: (pageState: CanvasesPageState) => void;
  };
};

export const createCanvasSlice: StateCreator<CanvasSlice> = (set) => ({
  canvas: {
    currentCanvasId: "",
    activeNotebookId: "",
    canvasPageState: CanvasesPageState.LIST,
    setCurrentCanvasId: (canvasId) =>
      set((state) => ({ canvas: { ...state.canvas, currentCanvasId: canvasId } })),
    setActiveNotebookId: (notebookId) =>
      set((state) => ({ canvas: { ...state.canvas, activeNotebookId: notebookId } })),
    setCanvasPageState: (pageState) =>
      set((state) => ({ canvas: { ...state.canvas, canvasPageState: pageState } })),
  },
});
