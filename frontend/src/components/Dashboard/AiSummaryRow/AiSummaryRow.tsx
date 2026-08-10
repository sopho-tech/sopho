import { AiSummaryDto, useDashboardSummary } from "src/api/ai_summary";
import { Box, Flex, Icon, Text } from "src/components/design-system";
import { formatRelativeTime, formatTimestamp } from "src/utils/timestamp_utils";

const SUMMARY_STYLE = { lineHeight: "var(--line-height-relaxed)" };

type AiSummaryRowProps = {
  dashboardId: string;
};

function StatusNote({
  iconColor,
  message,
  highlighted = false,
}: {
  iconColor: "warning" | "error";
  message: string;
  highlighted?: boolean;
}) {
  return (
    <Flex
      direction="row"
      gap="2xs"
      alignItems="center"
      alignSelf="flex-start"
      backgroundColor={highlighted ? "warning" : "default"}
      borderRadius="md"
      paddingX={highlighted ? "xs" : undefined}
      paddingY={highlighted ? "2xs" : undefined}
    >
      <Icon type="triangle_alert" color={iconColor} size="sm" />
      <Text as="span" fontSize="xs" color="subtle">
        {message}
      </Text>
    </Flex>
  );
}

function failureMessage(summary: AiSummaryDto) {
  return summary.error_message
    ? `Could not generate a summary: ${summary.error_message}`
    : "Could not generate a summary.";
}

export function AiSummaryRow({ dashboardId }: AiSummaryRowProps) {
  const { data: summary } = useDashboardSummary(dashboardId);

  if (!summary) {
    return null;
  }

  const isGenerating = summary.status === "GENERATING";
  const hasFailed = summary.status === "FAILED";

  if (!isGenerating && !hasFailed && !summary.summary_text) {
    return null;
  }

  return (
    <Flex direction="column" gap="2xs" borderRadius="lg" revealChildrenOnHover>
      <Box revealOnHover>
        <Text as="span" fontSize="xs" color="subtle">
          AI summary
        </Text>
      </Box>

      {isGenerating && (
        <StatusNote
          iconColor="warning"
          message="Generating a new summary. Showing the previous one until it is ready."
          highlighted
        />
      )}

      {hasFailed && (
        <StatusNote iconColor="error" message={failureMessage(summary)} />
      )}

      {summary.is_prompt_stale && !isGenerating && (
        <StatusNote
          iconColor="warning"
          message="Prompt changed since this summary was generated."
        />
      )}

      {summary.summary_text && (
        <Box sx={SUMMARY_STYLE}>
          <Text as="p" fontSize="base">
            {summary.summary_text}
          </Text>
        </Box>
      )}

      {summary.generated_at && (
        <Box revealOnHover>
          <Text as="span" fontSize="xs" color="subtle">
            {`Generated ${formatTimestamp(summary.generated_at)} · ${formatRelativeTime(summary.generated_at)}`}
          </Text>
        </Box>
      )}
    </Flex>
  );
}
