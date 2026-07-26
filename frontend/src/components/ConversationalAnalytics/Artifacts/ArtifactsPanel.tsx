import {
  Flex,
  IconButton,
  MotionFlex,
  Text,
} from "src/components/design-system";
import { Duration, EASE } from "src/components/design-system/animation";
import { CanvasGeneratedCard } from "src/components/ConversationalAnalytics/CanvasGeneratedCard";
import { useArtifacts } from "src/components/ConversationalAnalytics/Artifacts/ArtifactsContext";
import {
  ArtifactType,
  type Artifact,
} from "src/components/ConversationalAnalytics/dto";

const PANEL_STYLE = { flexShrink: 0 };
const PANEL_INITIAL = { x: 16, opacity: 0 };
const PANEL_ANIMATE = { x: 0, opacity: 1 };
const PANEL_TRANSITION = { duration: Duration.FAST, ease: EASE };
const CLOSE_TOOLTIP = { text: "Close artifacts" };

const renderArtifact = (artifact: Artifact) => {
  switch (artifact.type) {
    case ArtifactType.Canvas:
      return <CanvasGeneratedCard data={artifact.data} />;
    default:
      return null;
  }
};

export const ArtifactsPanel = () => {
  const { artifacts, isPanelOpen, closePanel } = useArtifacts();

  if (!isPanelOpen) return null;

  return (
    <MotionFlex
      as="aside"
      aria-label="Artifacts"
      direction="column"
      height="100%"
      width="22rem"
      borderLeft="divider"
      marginLeft="lg"
      paddingLeft="lg"
      paddingRight="sm"
      paddingTop="lg"
      paddingBottom="lg"
      gap="md"
      sx={PANEL_STYLE}
      initial={PANEL_INITIAL}
      animate={PANEL_ANIMATE}
      transition={PANEL_TRANSITION}
    >
      <Flex alignItems="center" justifyContent="space-between">
        <Text fontSize="lg" color="darkGrey">
          {`Artifacts (${artifacts.length})`}
        </Text>
        <IconButton
          type="close"
          backgroundColor="default"
          iconColor="grey"
          tooltip={CLOSE_TOOLTIP}
          onClick={closePanel}
        />
      </Flex>
      <Flex
        direction="column"
        gap="md"
        flex="grow"
        overflow="scrollY"
        paddingTop="sm"
        paddingRight="md"
      >
        {artifacts.map((artifact) => (
          <Flex key={artifact.id} direction="column">
            {renderArtifact(artifact)}
          </Flex>
        ))}
      </Flex>
    </MotionFlex>
  );
};
