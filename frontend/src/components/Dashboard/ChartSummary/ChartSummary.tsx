import { AiSummaryDto } from "src/api/ai_summary";
import {
  Box,
  Flex,
  Icon,
  IconButton,
  Popover,
  Text,
} from "src/components/design-system";
import { formatRelativeTime, formatTimestamp } from "src/utils/timestamp_utils";

const SUMMARY_STYLE = { lineHeight: "var(--line-height-relaxed)" };

const SUMMARY_TOOLTIP = {
  text: "view summary",
  direction: "top",
} as const;

type ChartSummaryProps = {
  summary: AiSummaryDto;
};

function SummaryBody({ summary }: ChartSummaryProps) {
  if (summary.status === "FAILED" && !summary.summary_text) {
    return (
      <Text as="p" fontSize="sm" color="subtle">
        {summary.error_message
          ? `Could not generate a summary: ${summary.error_message}`
          : "Could not generate a summary."}
      </Text>
    );
  }

  if (!summary.summary_text) {
    return null;
  }

  return (
    <Box sx={SUMMARY_STYLE}>
      <Text as="p" fontSize="base">
        {summary.summary_text}
      </Text>
    </Box>
  );
}

export function ChartSummary({ summary }: ChartSummaryProps) {
  const isGenerating = summary.status === "GENERATING";
  const hasContent = !!summary.summary_text || summary.status === "FAILED";

  if (!hasContent && !isGenerating) {
    return null;
  }

  return (
    <Popover>
      <Popover.Trigger>
        <IconButton
          type="file_text"
          backgroundColor="default"
          iconColor={isGenerating ? "warning" : "grey"}
          iconSize="sm"
          tooltip={SUMMARY_TOOLTIP}
        />
      </Popover.Trigger>
      <Popover.Content>
        <Flex direction="column" gap="2xs">
          {isGenerating && (
            <Flex direction="row" gap="2xs" alignItems="center">
              <Icon type="triangle_alert" color="warning" size="sm" />
              <Text as="span" fontSize="xs" color="subtle">
                Generating a new summary.
              </Text>
            </Flex>
          )}

          {summary.is_prompt_stale && !isGenerating && (
            <Flex direction="row" gap="2xs" alignItems="center">
              <Icon type="triangle_alert" color="warning" size="sm" />
              <Text as="span" fontSize="xs" color="subtle">
                Prompt changed since this summary was generated.
              </Text>
            </Flex>
          )}

          <SummaryBody summary={summary} />

          {summary.generated_at && (
            <Text as="span" fontSize="xs" color="subtle">
              {`Generated ${formatTimestamp(summary.generated_at)} · ${formatRelativeTime(summary.generated_at)}`}
            </Text>
          )}
        </Flex>
      </Popover.Content>
    </Popover>
  );
}
