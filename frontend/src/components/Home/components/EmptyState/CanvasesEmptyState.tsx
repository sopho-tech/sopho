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
      description="Create a canvas to build notebooks, dashboards and explore data"
      buttonLabel="New Canvas"
      onButtonClick={onCreateCanvas}
    />
  );
}
