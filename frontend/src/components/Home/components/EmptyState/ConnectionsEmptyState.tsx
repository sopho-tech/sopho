import { EmptyState } from "./EmptyState";

type ConnectionsEmptyStateProps = {
  onCreateConnection: () => void;
};

export function ConnectionsEmptyState({
  onCreateConnection,
}: ConnectionsEmptyStateProps) {
  return (
    <EmptyState
      icon="plug"
      heading="No connections yet"
      description="Connect your data sources to build notebooks, dashboards and explore data"
      buttonLabel="New Connection"
      onButtonClick={onCreateConnection}
    />
  );
}
