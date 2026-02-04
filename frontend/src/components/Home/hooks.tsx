import { useCallback } from "react";
import { useNavigate } from "react-router";
import { useCanvasStore } from "src/components/Canvases/store";
import { CanvasesPageState } from "src/components/Canvases/dto";
import { useConnectionsStore } from "src/components/Connection/store";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { useAllCanvases } from "src/api/canvas/queries";

export function useHomeNavigation() {
  const navigate = useNavigate();
  const { setCanvasPageState } = useCanvasStore();
  const { setConnectionDetailsPageState } = useConnectionsStore();

  const handleCreateCanvas = useCallback(() => {
    navigate(APP_ROUTES.CANVASES);
    setCanvasPageState(CanvasesPageState.CREATE_CANVAS_DIALOG);
  }, [navigate, setCanvasPageState]);

  const handleCreateConnection = useCallback(() => {
    navigate(APP_ROUTES.SETTINGS);
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.NEW);
  }, [navigate, setConnectionDetailsPageState]);

  return {
    handleCreateCanvas,
    handleCreateConnection,
  };
}

export function useRecentlyUpdatedCanvases(page: number = 0, pageSize: number = 9) {
  return useAllCanvases(page, pageSize);
}
