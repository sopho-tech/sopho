import { useEffect, useCallback } from "react";
import {
  useUpdateDashboard,
  useDashboardByCanvasId,
} from "src/api/dashboard/queries";
import { useStore, DashboardMode } from "src/store";
import { convertRGLayoutToDto } from "src/components/Dashboard/dto";
import { DashboardDto } from "src/components/Dashboard/dto";

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
