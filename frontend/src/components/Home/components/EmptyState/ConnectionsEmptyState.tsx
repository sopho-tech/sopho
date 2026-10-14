import { EmptyState } from "src/components/EmptyState";
import { useTenantPermission } from "src/api/tenant/queries";
import { PERMISSIONS } from "src/constants/permissions";

type ConnectionsEmptyStateProps = {
  onCreateConnection: () => void;
};

const HEADING = "No connections yet";

export function ConnectionsEmptyState({
  onCreateConnection,
}: ConnectionsEmptyStateProps) {
  const canCreateConnection = useTenantPermission(
    PERMISSIONS.CONNECTION_CREATE,
  );

  if (!canCreateConnection) {
    return (
      <EmptyState
        icon="plug"
        heading={HEADING}
        description="A workspace admin can connect data sources for everyone to use"
      />
    );
  }

  return (
    <EmptyState
      icon="plug"
      heading={HEADING}
      description="Connect your data sources to build notebooks, dashboards and explore data"
      buttonLabel="New Connection"
      onButtonClick={onCreateConnection}
    />
  );
}
