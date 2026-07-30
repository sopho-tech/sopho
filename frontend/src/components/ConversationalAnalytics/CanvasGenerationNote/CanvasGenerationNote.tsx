import { Box, Flex, Text } from "src/components/design-system";
import type { CanvasGeneratedData } from "src/components/ConversationalAnalytics/dto";

const NOTE_STYLE = { lineHeight: "var(--line-height-relaxed)" };

type CanvasGenerationNoteProps = {
  data: CanvasGeneratedData;
};

function changeCounts(data: CanvasGeneratedData): string {
  return [
    { count: data.cells_added, label: "added" },
    { count: data.cells_updated, label: "updated" },
    { count: data.cells_removed, label: "removed" },
  ]
    .filter((part) => (part.count ?? 0) > 0)
    .map((part) => `${part.count} ${part.label}`)
    .join(" · ");
}

export const CanvasGenerationNote = ({ data }: CanvasGenerationNoteProps) => {
  const reasoning = data.reasoning?.trim();
  if (!reasoning) return null;

  const outcome = data.reused ? "Updated existing canvas" : "Created a new canvas";
  const counts = changeCounts(data);

  return (
    <Box width="100%" paddingX="md" paddingY="sm" sx={NOTE_STYLE}>
      <Flex direction="column" gap="2xs">
        <Text fontSize="sm" color="subtle">
          {counts ? `${outcome} · ${counts}` : `${outcome} · no changes`}
        </Text>
        <Text as="p" fontSize="base">
          {reasoning}
        </Text>
      </Flex>
    </Box>
  );
};
