import { beforeEach, describe, expect, test } from "vitest";
import {
  DEFAULT_CHART_HEIGHT,
  DEFAULT_CHART_WIDTH,
  type DashboardDto,
} from "src/components/Dashboard/dto";
import { DashboardMode, useStore } from "src/store";
import { CHART_CELL, NOTEBOOK_ID } from "src/test/test_data";

const initialStoreState = useStore.getState();

function dashboardWithLayout(layout: DashboardDto["layout"]): DashboardDto {
  return {
    id: "2f3a4b5c-6d7e-4f80-91a2-b3c4d5e6f708",
    name: "Sales",
    description: "Sales dashboard",
    status: "ACTIVE",
    layout,
  };
}

describe("dashboard slice", () => {
  beforeEach(() => {
    useStore.setState(initialStoreState, true);
  });

  test("converts the backend layout into grid items", () => {
    useStore.getState().dashboard.resetFromBackendData(
      dashboardWithLayout([
        {
          cell_id: CHART_CELL.id,
          notebook_id: NOTEBOOK_ID,
          x_position: 2,
          y_position: 1,
          x_size: 6,
          y_size: 4,
        },
      ]),
    );

    expect(useStore.getState().dashboard.layout).toEqual([
      {
        i: CHART_CELL.id,
        x: 2,
        y: 1,
        w: 6,
        h: 4,
        minW: DEFAULT_CHART_WIDTH,
        minH: DEFAULT_CHART_HEIGHT,
      },
    ]);
  });

  test("keeps the current layout when the backend has none", () => {
    const dashboard = useStore.getState().dashboard;
    dashboard.setLayout([{ i: CHART_CELL.id, x: 0, y: 0, w: 4, h: 3 }]);

    dashboard.resetFromBackendData(dashboardWithLayout(undefined));

    expect(useStore.getState().dashboard.layout).toHaveLength(1);
  });

  test("tracks save requests and the editing mode", () => {
    const dashboard = useStore.getState().dashboard;
    dashboard.setMode(DashboardMode.EDITING);
    dashboard.requestSave();
    expect(useStore.getState().dashboard.mode).toBe(DashboardMode.EDITING);
    expect(useStore.getState().dashboard.saveRequested).toBe(true);

    useStore.getState().dashboard.clearSaveRequest();
    expect(useStore.getState().dashboard.saveRequested).toBe(false);
  });
});
