import { EmptyState } from "src/components/EmptyState";

type CanvasesEmptyStateProps = {
  onCreateCanvas: () => void;
};

export function CanvasesEmptyState({
  onCreateCanvas,
}: CanvasesEmptyStateProps) {
  return (
    <EmptyState
      icon="layers"
      heading="No canvases yet"
      description="Create a canvas to explore data and build dashboards"
      buttonLabel="New Canvas"
      onButtonClick={onCreateCanvas}
    />
  );
}
