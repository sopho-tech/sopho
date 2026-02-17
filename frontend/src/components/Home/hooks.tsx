import { useCallback } from "react";
import { useNavigate } from "react-router";
import { useStore } from "src/store";
import { CanvasesPageState } from "src/components/Canvases/dto";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { useAllCanvases } from "src/api/canvas/queries";
import { useConnections } from "src/api/connection";

export function useHomeNavigation() {
  const navigate = useNavigate();
  const setCanvasPageState = useStore(
    (state) => state.canvas.setCanvasPageState
  );
  const setConnectionDetailsPageState = useStore(
    (state) => state.connection.setConnectionDetailsPageState
  );

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

export function useRecentlyUpdatedCanvases(
  page: number = 0,
  pageSize: number = 9
) {
  return useAllCanvases(page, pageSize);
}

export const useIsConnectionsEmptyState = (): boolean | undefined => {
  const connectionsQuery = useConnections();
  return connectionsQuery.data?.length === 0;
};

export const useIsCanvasesEmptyState = (): boolean | undefined => {
  const canvasesQuery = useAllCanvases();
  return canvasesQuery.data?.totalItems === 0;
};

export const useIsEmptyState = (): boolean | undefined => {
  const isConnectionsEmptyState = useIsConnectionsEmptyState();
  const isCanvasesEmptyState = useIsCanvasesEmptyState();
  return isConnectionsEmptyState || isCanvasesEmptyState;
};
