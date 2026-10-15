import {
  useHomeNavigation,
  useIsCanvasesEmptyState,
  useIsConnectionsEmptyState,
} from "src/components/Home/hooks";
import { CanvasesEmptyState } from "src/components/Home/components/EmptyState/CanvasesEmptyState";
import { ConnectionsEmptyState } from "src/components/Home/components/EmptyState/ConnectionsEmptyState";

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
