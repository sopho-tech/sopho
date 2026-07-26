import { IconButton } from "src/components/design-system";
import { useArtifacts } from "src/components/ConversationalAnalytics/Artifacts/ArtifactsContext";

const ARTIFACTS_TOOLTIP = { text: "Artifacts" };

export const ArtifactsButton = () => {
  const { artifacts, togglePanel } = useArtifacts();

  if (artifacts.length === 0) return null;

  return (
    <IconButton
      type="layers"
      backgroundColor="default"
      iconColor="grey"
      tooltip={ARTIFACTS_TOOLTIP}
      onClick={togglePanel}
    />
  );
};
