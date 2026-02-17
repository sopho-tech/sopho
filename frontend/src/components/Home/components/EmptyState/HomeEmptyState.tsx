import {
  useHomeNavigation,
  useIsCanvasesEmptyState,
  useIsConnectionsEmptyState,
} from "../../hooks";
import { CanvasesEmptyState } from "./CanvasesEmptyState";
import { ConnectionsEmptyState } from "./ConnectionsEmptyState";

export function HomeEmptyState() {
  const { handleCreateCanvas, handleCreateConnection } = useHomeNavigation();
  const isConnectionsEmptyState = useIsConnectionsEmptyState();
  const isCanvasesEmptyState = useIsCanvasesEmptyState();

  if (isConnectionsEmptyState) {
    return (
      <ConnectionsEmptyState onCreateConnection={handleCreateConnection} />
    );
  }
  if (isCanvasesEmptyState) {
    return <CanvasesEmptyState onCreateCanvas={handleCreateCanvas} />;
  }
  return null;
}
