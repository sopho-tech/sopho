import { useMemo } from "react";
import { Flex, Box, ShimmeringText, Text } from "src/components/design-system";
import {
  IN_PROGRESS_EVENTS,
  extractChartData,
  extractCanvasGenerated,
  extractNarration,
  type AgentEvent,
} from "src/components/ConversationalAnalytics/dto";
import { CanvasGeneratedCard } from "src/components/ConversationalAnalytics/CanvasGeneratedCard";
import { CanvasGenerationNote } from "src/components/ConversationalAnalytics/CanvasGenerationNote";
import { ResultNarration } from "src/components/ConversationalAnalytics/ResultNarration";
import {
  STEP_LABELS,
  TRACE_HIDDEN_EVENTS,
  TraceStep,
  RouterDecisionBubble,
  extractRouterDecision,
} from "src/components/ConversationalAnalytics/AgentTrace/trace-steps";
import { QueryResultChart } from "src/components/ConversationalAnalytics/QueryResultChart";

type DisplayStep = {
  event: AgentEvent;
  isActive: boolean;
};

function buildDisplaySteps(events: AgentEvent[]): DisplayStep[] {
  const steps: DisplayStep[] = [];

  for (const event of events) {
    if (TRACE_HIDDEN_EVENTS.has(event.event_name)) continue;
    if (IN_PROGRESS_EVENTS.has(event.event_name)) continue;
    steps.push({ event, isActive: false });
  }

  let lastNonMeta: AgentEvent | undefined;
  for (let i = events.length - 1; i >= 0; i--) {
    if (!TRACE_HIDDEN_EVENTS.has(events[i].event_name)) {
      lastNonMeta = events[i];
      break;
    }
  }

  if (lastNonMeta && IN_PROGRESS_EVENTS.has(lastNonMeta.event_name)) {
    steps.push({ event: lastNonMeta, isActive: true });
  }

  return steps;
}

function deriveStatusText(events: AgentEvent[]): string {
  for (let i = events.length - 1; i >= 0; i--) {
    if (IN_PROGRESS_EVENTS.has(events[i].event_name)) {
      return STEP_LABELS[events[i].event_name] ?? "Processing";
    }
  }
  return "Analyzing";
}

type StreamingAgentTraceProps = {
  events: AgentEvent[];
  isStreaming: boolean;
};

export function StreamingAgentTrace({
  events,
  isStreaming,
}: StreamingAgentTraceProps) {
  const steps = useMemo(() => buildDisplaySteps(events), [events]);
  const statusText = useMemo(() => deriveStatusText(events), [events]);
  const chartData = useMemo(() => extractChartData(events), [events]);
  const canvas = useMemo(() => extractCanvasGenerated(events), [events]);
  const routerDecision = useMemo(() => extractRouterDecision(events), [events]);
  const narration = useMemo(() => extractNarration(events), [events]);

  if (steps.length === 0 && !isStreaming && !routerDecision) return null;

  return (
    <>
      <Box width="100%">
        <Flex alignItems="center" gap="xs" paddingY="xs" paddingX="sm">
          {isStreaming ? (
            <ShimmeringText text={statusText} fontSize="sm" />
          ) : (
            <Text fontSize="sm" color="darkGrey">
              Explanation
            </Text>
          )}
        </Flex>
        {steps.length > 0 && (
          <Flex direction="column" paddingLeft="sm" paddingTop="sm">
            {steps.map((step, i) => (
              <TraceStep
                key={step.event.event_name}
                event={step.event}
                isLast={i === steps.length - 1}
                isActive={step.isActive}
              />
            ))}
          </Flex>
        )}
      </Box>
      {routerDecision && (
        <RouterDecisionBubble
          data={routerDecision.data}
          createdAt={routerDecision.createdAt}
        />
      )}
      {narration && <ResultNarration text={narration} />}
      {chartData && <QueryResultChart chartData={chartData} />}
      {canvas && (
        <>
          <CanvasGenerationNote data={canvas} />
          <CanvasGeneratedCard data={canvas} />
        </>
      )}
    </>
  );
}
