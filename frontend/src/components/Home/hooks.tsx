import { useCallback } from "react";
import { useNavigate } from "react-router";
import { useStore } from "src/store";
import { CanvasesPageState } from "src/components/Canvases/dto";
import { ConnectionDetailsPageStateEnum } from "src/components/Connection/dto";
import { APP_ROUTES } from "src/constants/app_routes";
import { useTenantPath } from "src/utils/tenant";
import { useAllCanvases } from "src/api/canvas/queries";
import { useConnections } from "src/api/connection";

export function useHomeNavigation() {
  const navigate = useNavigate();
  const toTenantPath = useTenantPath();
  const setCanvasPageState = useStore(
    (state) => state.canvas.setCanvasPageState,
  );
  const setConnectionDetailsPageState = useStore(
    (state) => state.connection.setConnectionDetailsPageState,
  );

  const handleCreateCanvas = useCallback(() => {
    void navigate(toTenantPath(APP_ROUTES.CANVASES));
    setCanvasPageState(CanvasesPageState.CREATE_CANVAS_DIALOG);
  }, [navigate, setCanvasPageState, toTenantPath]);

  const handleCreateConnection = useCallback(() => {
    void navigate(toTenantPath(APP_ROUTES.SETTINGS_ROUTES.CONNECTIONS));
    setConnectionDetailsPageState(ConnectionDetailsPageStateEnum.NEW);
  }, [navigate, setConnectionDetailsPageState, toTenantPath]);

  return {
    handleCreateCanvas,
    handleCreateConnection,
  };
}

export function useRecentlyUpdatedCanvases(
  page: number = 0,
  pageSize: number = 9,
) {
  return useAllCanvases(null, page, pageSize);
}

export const useIsConnectionsEmptyState = (): boolean | undefined => {
  const connectionsQuery = useConnections(null);
  return connectionsQuery.data?.length === 0;
};

export const useIsCanvasesEmptyState = (): boolean | undefined => {
  const canvasesQuery = useAllCanvases(null);
  return canvasesQuery.data?.totalItems === 0;
};

export const useIsEmptyState = (): boolean | undefined => {
  const isConnectionsEmptyState = useIsConnectionsEmptyState();
  const isCanvasesEmptyState = useIsCanvasesEmptyState();
  return isConnectionsEmptyState || isCanvasesEmptyState;
};
