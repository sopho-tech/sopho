import { Box, Text } from "src/components/design-system";

const NARRATION_STYLE = { lineHeight: "var(--line-height-relaxed)" };

type ResultNarrationProps = {
  text: string;
};

export const ResultNarration = ({ text }: ResultNarrationProps) => {
  const narration = text.trim();
  if (!narration) return null;

  return (
    <Box width="100%" paddingX="md" paddingY="sm" sx={NARRATION_STYLE}>
      <Text as="p" fontSize="base">
        {narration}
      </Text>
    </Box>
  );
};
