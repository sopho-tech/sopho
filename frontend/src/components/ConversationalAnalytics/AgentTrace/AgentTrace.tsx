import { useMemo, useState } from "react";
import {
  Icon,
  Text,
  Flex,
  Box,
  Collapsible,
  MotionFlex,
} from "src/components/design-system";
import { rotateToggleTransition } from "src/components/design-system/animation";
import {
  AgentEventName,
  extractCanvasGenerated,
  extractNarration,
  type AgentEvent,
  type ConversationMessageContentDto,
  type GeneratedSqlData,
  type RecommendedVisualizationData,
} from "src/components/ConversationalAnalytics/dto";
import { CanvasGeneratedCard } from "src/components/ConversationalAnalytics/CanvasGeneratedCard";
import { ResultNarration } from "src/components/ConversationalAnalytics/ResultNarration";
import {
  parseEvent,
  isCompletedEvent,
  TRACE_HIDDEN_EVENTS,
  TraceStep,
  RouterDecisionBubble,
  extractRouterDecision,
} from "./trace-steps";
import { QueryResultChartContainer } from "src/components/ConversationalAnalytics/QueryResultChart";
import styles from "./AgentTrace.module.css";

type AgentTraceProps = {
  connectionId: string;
  contents: ConversationMessageContentDto[];
};

export function AgentTrace({ connectionId, contents }: AgentTraceProps) {
  const [traceOpen, setTraceOpen] = useState(false);

  const events: AgentEvent[] = useMemo(
    () =>
      contents
        .map(parseEvent)
        .filter((e): e is AgentEvent => e !== null)
        .filter(isCompletedEvent),
    [contents],
  );

  const visibleEvents = useMemo(
    () => events.filter((e) => !TRACE_HIDDEN_EVENTS.has(e.event_name)),
    [events],
  );

  const routerDecision = useMemo(
    () => extractRouterDecision(events, contents),
    [events, contents],
  );

  const sql = useMemo(() => {
    const event = events.find(
      (e) => e.event_name === AgentEventName.GeneratedSql,
    );
    return event?.data ? (event.data as GeneratedSqlData).sql : "";
  }, [events]);

  const visualization = useMemo(() => {
    const event = events.find(
      (e) => e.event_name === AgentEventName.RecommendedVisualization,
    );
    return event?.data
      ? (event.data as RecommendedVisualizationData).visualization
      : null;
  }, [events]);

  const canvas = useMemo(() => extractCanvasGenerated(events), [events]);

  const narration = useMemo(() => extractNarration(events), [events]);

  if (events.length === 0 && !routerDecision) return null;

  return (
    <>
      {routerDecision && (
        <RouterDecisionBubble
          data={routerDecision.data}
          createdAt={routerDecision.createdAt}
        />
      )}
      {visibleEvents.length > 0 && (
        <Collapsible open={traceOpen} onOpenChange={setTraceOpen}>
          <Box width="100%">
            <Collapsible.Trigger>
              <Flex
                alignItems="center"
                gap="xs"
                paddingY="xs"
                paddingX="sm"
                borderRadius="lg"
                className={styles.traceToggle}
                sx={{
                  cursor: "pointer",
                  userSelect: "none",
                }}
              >
                <Flex
                  alignItems="center"
                  justifyContent="center"
                  flex="none"
                  width="calc(var(--space-md) + var(--space-2xs))"
                >
                  <MotionFlex
                    alignItems="center"
                    justifyContent="center"
                    animate={{ rotate: traceOpen ? 90 : 0 }}
                    transition={rotateToggleTransition}
                  >
                    <Icon
                      type="chevron_right"
                      color="grey"
                      size="sm"
                      interactive={false}
                    />
                  </MotionFlex>
                </Flex>
                <Text fontSize="sm" color="darkGrey">
                  Explanation
                </Text>
              </Flex>
            </Collapsible.Trigger>
            <Collapsible.Content>
              <Flex direction="column" paddingLeft="sm" paddingTop="sm">
                {visibleEvents.map((event, i) => (
                  <TraceStep
                    key={i}
                    event={event}
                    isLast={i === visibleEvents.length - 1}
                  />
                ))}
              </Flex>
            </Collapsible.Content>
          </Box>
        </Collapsible>
      )}
      {narration && <ResultNarration text={narration} />}
      {sql && visualization && (
        <QueryResultChartContainer
          connectionId={connectionId}
          sql={sql}
          visualization={visualization}
        />
      )}
      {canvas && <CanvasGeneratedCard data={canvas} />}
    </>
  );
}
