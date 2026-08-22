import { create } from "zustand";
import { createDashboardSlice, DashboardSlice } from "./dashboardSlice";
import { createCanvasSlice, CanvasSlice } from "./canvasSlice";
import { createNotebookSlice, NotebookSlice } from "./notebookSlice";
import { createCellSlice, CellSlice } from "./cellSlice";
import { createChartCellSlice, ChartCellSlice } from "./chartCellSlice";
import { createConnectionSlice, ConnectionSlice } from "./connectionSlice";
import { createCommandMenuSlice, CommandMenuSlice } from "./commandMenuSlice";

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
