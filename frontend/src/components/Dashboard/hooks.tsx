import { useEffect, useCallback, useMemo } from "react";
import {
  useUpdateDashboard,
  useDashboardByCanvasId,
} from "src/api/dashboard/queries";
import { useStore, DashboardMode } from "src/store";
import { convertRGLayoutToDto } from "src/components/Dashboard/dto";
import { DashboardDto } from "src/components/Dashboard/dto";
import { useHandleExecuteCell } from "src/components/Notebook/Cell";
import { ExecutionState } from "src/components/Notebook/Cell/dto";

export function useDashboardSave(dashboardData: DashboardDto | undefined) {
  const activeNotebookId = useStore((state) => state.canvas.activeNotebookId);
  const saveRequested = useStore((state) => state.dashboard.saveRequested);
  const layout = useStore((state) => state.dashboard.layout);
  const setMode = useStore((state) => state.dashboard.setMode);
  const clearSaveRequest = useStore(
    (state) => state.dashboard.clearSaveRequest
  );
  const updateDashboardMutation = useUpdateDashboard();

  useEffect(() => {
    if (!saveRequested) {
      return;
    }

    setMode(DashboardMode.VIEWING);
    clearSaveRequest();

    if (!dashboardData || !activeNotebookId) {
      return;
    }

    const layoutDto = convertRGLayoutToDto(layout, activeNotebookId);
    updateDashboardMutation.mutate(
      {
        dashboardId: dashboardData.id,
        payload: {
          ...dashboardData,
          layout: layoutDto,
        },
      },
      {
        onError: () => {
          setMode(DashboardMode.EDITING);
        },
      }
    );
  }, [
    saveRequested,
    dashboardData,
    activeNotebookId,
    layout,
    setMode,
    clearSaveRequest,
    updateDashboardMutation,
  ]);

  return {
    isSaving: updateDashboardMutation.isPending,
  };
}

export function useRefreshDashboardCharts() {
  const layout = useStore((state) => state.dashboard.layout);
  const handleExecuteCell = useHandleExecuteCell();

  const chartCellIds = useMemo(
    () => layout.map((item) => item.i).filter((id) => !id.startsWith("_")),
    [layout]
  );

  const isRefreshing = useStore((state) =>
    chartCellIds.some(
      (cellId) => state.cell.executionStates[cellId] === ExecutionState.RUNNING
    )
  );

  const refreshAll = useCallback(() => {
    if (isRefreshing) {
      return;
    }
    chartCellIds.forEach((cellId) => handleExecuteCell(cellId));
  }, [chartCellIds, handleExecuteCell, isRefreshing]);

  return { refreshAll, isRefreshing, chartCount: chartCellIds.length };
}

export function useDashboardReset(canvasId: string, isDashboardView: boolean) {
  const mode = useStore((state) => state.dashboard.mode);
  const setMode = useStore((state) => state.dashboard.setMode);
  const requestSave = useStore((state) => state.dashboard.requestSave);
  const resetFromBackendData = useStore(
    (state) => state.dashboard.resetFromBackendData
  );
  const setShowChartBrowser = useStore(
    (state) => state.dashboard.setShowChartBrowser
  );
  const dashboardQuery = useDashboardByCanvasId(canvasId);
  const isEditing = mode === DashboardMode.EDITING;

  const handleCancelClick = useCallback(async () => {
    if (isDashboardView) {
      const { data } = await dashboardQuery.refetch();
      if (data) {
        resetFromBackendData(data);
      }
    }
    setShowChartBrowser(false);
    setMode(DashboardMode.VIEWING);
  }, [
    isDashboardView,
    dashboardQuery,
    resetFromBackendData,
    setShowChartBrowser,
    setMode,
  ]);

  const handleEditSaveClick = useCallback(async () => {
    if (isEditing) {
      requestSave();
      setShowChartBrowser(false);
    } else {
      if (isDashboardView) {
        const { data } = await dashboardQuery.refetch();
        if (data) {
          resetFromBackendData(data);
        }
      }
      setMode(DashboardMode.EDITING);
      setShowChartBrowser(true);
    }
  }, [
    isEditing,
    isDashboardView,
    dashboardQuery,
    resetFromBackendData,
    requestSave,
    setShowChartBrowser,
    setMode,
  ]);

  return {
    handleCancelClick,
    handleEditSaveClick,
  };
}
