import { create } from "zustand";
import { CanvasesPageState } from "src/components/Canvases/dto";

interface CanvasStore {
  currentCanvasId: string;
  activeNotebookId: string;
  canvasPageState: CanvasesPageState;
  setCurrentCanvasId: (canvasId: string) => void;
  setActiveNotebookId: (notebookId: string) => void;
  setCanvasPageState: (pageState: CanvasesPageState) => void;
}

export const useCanvasStore = create<CanvasStore>((set) => ({
  currentCanvasId: "",
  activeNotebookId: "",
  canvasPageState: CanvasesPageState.LIST,
  setCurrentCanvasId: (canvasId) => set({ currentCanvasId: canvasId }),
  setActiveNotebookId: (notebookId) => set({ activeNotebookId: notebookId }),
  setCanvasPageState: (pageState) =>
    set(() => ({ canvasPageState: pageState })),
}));
