import { EmptyState } from "src/components/EmptyState";
import { useTenantPermission } from "src/api/tenant/queries";
import { PERMISSIONS } from "src/constants/permissions";

type CanvasesEmptyStateProps = {
  onCreateCanvas: () => void;
};

const HEADING = "No canvases yet";

export function CanvasesEmptyState({
  onCreateCanvas,
}: CanvasesEmptyStateProps) {
  const canCreateCanvas = useTenantPermission(PERMISSIONS.CANVAS_CREATE);

  if (!canCreateCanvas) {
    return (
      <EmptyState
        icon="layers"
        heading={HEADING}
        description="Canvases shared with you will appear here"
      />
    );
  }

  return (
    <EmptyState
      icon="layers"
      heading={HEADING}
      description="Create a canvas to explore data and build dashboards"
      buttonLabel="New Canvas"
      onButtonClick={onCreateCanvas}
    />
  );
}
