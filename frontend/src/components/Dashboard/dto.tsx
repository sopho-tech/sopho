import { Layout } from "react-grid-layout";

export const DEFAULT_CHART_WIDTH = 4;
export const DEFAULT_CHART_HEIGHT = 3;

export type LayoutDto = {
  cell_id: string;
  notebook_id: string;
  x_position: number;
  y_position: number;
  x_size: number;
  y_size: number;
};

export type DashboardDto = {
  id: string;
  name?: string;
  description?: string;
  layout?: LayoutDto[];
  status: "ACTIVE" | "INACTIVE";
};

export function convertRGLayoutToDto(
  layout: Layout[],
  notebookId: string
): LayoutDto[] {
  return layout.map((item) => ({
    cell_id: item.i,
    notebook_id: notebookId,
    x_position: item.x,
    y_position: item.y,
    x_size: item.w,
    y_size: item.h,
  }));
}

export function convertDtoToRGLayout(layoutDto: LayoutDto[]): Layout[] {
  return layoutDto.map((item) => ({
    i: item.cell_id,
    x: item.x_position,
    y: item.y_position,
    w: item.x_size,
    h: item.y_size,
    minW: DEFAULT_CHART_WIDTH,
    minH: DEFAULT_CHART_HEIGHT,
  }));
}
