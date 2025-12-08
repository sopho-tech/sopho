import { useEffect, useCallback } from "react";
import { useUpdateDashboard } from "src/api/dashboard/queries";
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

export function useSaveDashboard(dashboardData: DashboardDto | undefined) {
  const { activeNotebookId } = useCanvasStore();
  const { layout, setMode } = useDashboardStore();
  const updateDashboardMutation = useUpdateDashboard();

  const save = useCallback(() => {
    setMode(DashboardMode.VIEWING);

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
  }, [dashboardData, activeNotebookId, layout, setMode, updateDashboardMutation]);

  return {
    save,
    isSaving: updateDashboardMutation.isPending,
  };
}
