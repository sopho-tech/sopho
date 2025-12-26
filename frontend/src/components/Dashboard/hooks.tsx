import { useEffect, useCallback } from "react";
import {
  useUpdateDashboard,
  useDashboardByCanvasId,
} from "src/api/dashboard/queries";
import { useCanvasStore } from "src/components/Canvases/store";
import {
  useDashboardStore,
  DashboardMode,
} from "src/components/Dashboard/store";
import { convertRGLayoutToDto } from "src/components/Dashboard/dto";
import { DashboardDto } from "src/components/Dashboard/dto";

export function useDashboardSave(dashboardData: DashboardDto | undefined) {
  const { activeNotebookId } = useCanvasStore();
  const { layout, saveRequested, setMode, clearSaveRequest } =
    useDashboardStore();
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
  const {
    mode,
    setMode,
    requestSave,
    resetFromBackendData,
    setShowChartBrowser,
  } = useDashboardStore();
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
  }, [isDashboardView, dashboardQuery, resetFromBackendData, setMode]);

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
    setMode,
    requestSave,
  ]);

  return {
    handleCancelClick,
    handleEditSaveClick,
  };
}
