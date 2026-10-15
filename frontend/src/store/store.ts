import { create } from "zustand";
import { createDashboardSlice, DashboardSlice } from "src/store/dashboardSlice";
import { createCanvasSlice, CanvasSlice } from "src/store/canvasSlice";
import { createNotebookSlice, NotebookSlice } from "src/store/notebookSlice";
import { createCellSlice, CellSlice } from "src/store/cellSlice";
import { createChartCellSlice, ChartCellSlice } from "src/store/chartCellSlice";
import {
  createConnectionSlice,
  ConnectionSlice,
} from "src/store/connectionSlice";
import {
  createCommandMenuSlice,
  CommandMenuSlice,
} from "src/store/commandMenuSlice";

export type Store = DashboardSlice &
  CanvasSlice &
  NotebookSlice &
  CellSlice &
  ChartCellSlice &
  ConnectionSlice &
  CommandMenuSlice;

export const useStore = create<Store>()((...a) => ({
  ...createDashboardSlice(...a),
  ...createCanvasSlice(...a),
  ...createNotebookSlice(...a),
  ...createCellSlice(...a),
  ...createChartCellSlice(...a),
  ...createConnectionSlice(...a),
  ...createCommandMenuSlice(...a),
}));
