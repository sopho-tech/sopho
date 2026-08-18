import { useState } from "react";
import { AiSummaryDto, useDashboardSummary } from "src/api/ai_summary";
import {
  Box,
  Collapsible,
  Flex,
  Icon,
  MotionFlex,
  Text,
} from "src/components/design-system";
import { rotateToggleTransition } from "src/components/design-system/animation";
import { formatRelativeTime, formatTimestamp } from "src/utils/timestamp_utils";

const SUMMARY_STYLE = { lineHeight: "var(--line-height-relaxed)" };

const CHEVRON_COLLAPSED = { rotate: 0 };
const CHEVRON_EXPANDED = { rotate: 90 };

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
  const [expanded, setExpanded] = useState(false);

  if (!summary) {
    return null;
  }

  const isGenerating = summary.status === "GENERATING";
  const hasFailed = summary.status === "FAILED";

  if (!isGenerating && !hasFailed && !summary.summary_text) {
    return null;
  }

  return (
    <Collapsible open={expanded} onOpenChange={setExpanded}>
      <Collapsible.Trigger asChild={false}>
        <MotionFlex
          alignItems="center"
          justifyContent="center"
          flex="none"
          animate={expanded ? CHEVRON_EXPANDED : CHEVRON_COLLAPSED}
          transition={rotateToggleTransition}
        >
          <Icon
            type="chevron_right"
            color="grey"
            size="sm"
            interactive={false}
          />
        </MotionFlex>
        <Text as="span" fontSize="xs" color="subtle">
          AI summary
        </Text>
      </Collapsible.Trigger>

      <Collapsible.Content>
        <Flex
          direction="column"
          gap="2xs"
          paddingTop="2xs"
          paddingLeft="md"
          borderRadius="lg"
        >
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
            <Text as="span" fontSize="xs" color="subtle">
              {`Generated ${formatTimestamp(summary.generated_at)} · ${formatRelativeTime(summary.generated_at)}`}
            </Text>
          )}
        </Flex>
      </Collapsible.Content>
    </Collapsible>
  );
}
