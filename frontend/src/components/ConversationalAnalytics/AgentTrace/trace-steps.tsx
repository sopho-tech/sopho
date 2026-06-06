import { Fragment, useState } from "react";
import {
  Icon,
  Text,
  Flex,
  Box,
  Collapsible,
  MotionFlex,
  Spinner,
  Badge,
} from "src/components/design-system";
import type { BadgeProps } from "src/components/design-system/Badge/Badge";
import { rotateToggleTransition } from "src/components/design-system/animation";
import type {
  IconType,
  IconColor,
} from "src/components/design-system/datatypes";
import {
  SophoTable,
  type ColumnConfig,
} from "src/components/SophoTable/SophoTable";
import {
  AgentEventName,
  IN_PROGRESS_EVENTS,
  type AgentEvent,
  type ConversationMessageContentDto,
  type GeneratedCandidateHypothesisData,
  type IntegratedCandidatePlansData,
  type ExecutedSearchSpaceReductionData,
  type ExecutedFunctionalRoleAnalysisData,
  type ExecutedDataProfilingData,
  type ExecutedSchemaLinkingSynthesisData,
  type GeneratedSqlData,
  type DataCatalogDatabase,
  type EmpericalObservation,
  type EmpericalObservationColumn,
  type SchemaLinkingFinalSynthesisResponse,
  type SchemaLinkingRejectedCandidate,
  RouterCode,
  type RouterDecisionData,
} from "src/components/ConversationalAnalytics/dto";
import { MessageHoverFooter } from "src/components/ConversationalAnalytics/ExistingConversationPanel/MessageHoverFooter";
import panelStyles from "src/components/ConversationalAnalytics/ExistingConversationPanel/ExistingConversationPanel.module.css";
import styles from "./AgentTrace.module.css";

const TRACE_STEP_ICON: IconType = "circle_dot";
const TRACE_STEP_ICON_COLOR: IconColor = "grey";

export const STEP_LABELS: Record<string, string> = {
  [AgentEventName.Routing]: "Analyzing your question…",
  [AgentEventName.Routed]: "Routed",
  [AgentEventName.GeneratedCandidateHypothesis]: "Generated Candidate Plans",
  [AgentEventName.IntegratedCandidatePlans]: "Integrated Master Plan",
  [AgentEventName.ExecutedSearchSpaceReduction]: "Selected Data Catalog",
  [AgentEventName.ExecutedFunctionalRoleAnalysis]: "Functional Role Analysis",
  [AgentEventName.ExecutedDataProfiling]: "Data Profiling",
  [AgentEventName.ExecutedSchemaLinkingSynthesis]: "Schema Linking Synthesis",
  [AgentEventName.GeneratedSql]: "Generated SQL",
  [AgentEventName.Error]: "Error",
  [AgentEventName.GeneratingCandidateHypothesis]: "Generating Candidate Plans",
  [AgentEventName.IntegratingCandidatePlans]: "Integrating Master Plan",
  [AgentEventName.ExecutingSearchSpaceReduction]: "Selecting Data Catalog",
  [AgentEventName.ExecutingFunctionalRoleAnalysis]:
    "Analyzing Functional Roles",
  [AgentEventName.ExecutingDataProfiling]: "Profiling Data",
  [AgentEventName.ExecutingSchemaLinkingSynthesis]: "Linking Schema",
  [AgentEventName.GeneratingSql]: "Generating SQL",
  [AgentEventName.ExecutedQuery]: "Executed Query",
  [AgentEventName.ExecutingQuery]: "Executing Query",
  [AgentEventName.RecommendingVisualization]: "Recommending Visualization",
  [AgentEventName.RecommendedVisualization]: "Recommended Visualization",
};

export const TRACE_HIDDEN_EVENTS = new Set<string>([
  AgentEventName.Starting,
  AgentEventName.Routing,
  AgentEventName.Routed,
  AgentEventName.Completed,
  AgentEventName.AwaitingClarification,
  AgentEventName.Rejected,
  AgentEventName.ExecutedQuery,
  AgentEventName.RecommendedVisualization,
  AgentEventName.SuggestedFollowups,
]);

const OBSERVATION_COLUMNS: ColumnConfig<EmpericalObservationColumn>[] = [
  {
    key: "column_name",
    header: "Column",
    type: "accessor",
    accessor: "column_name",
  },
  {
    key: "relevance_reason",
    header: "Reason",
    type: "accessor",
    accessor: "relevance_reason",
  },
  {
    key: "observations",
    header: "Observations",
    type: "accessor",
    accessor: "observations",
  },
];

const REJECTED_CANDIDATE_COLUMNS: ColumnConfig<SchemaLinkingRejectedCandidate>[] =
  [
    { key: "table", header: "Table", type: "accessor", accessor: "table" },
    { key: "column", header: "Column", type: "accessor", accessor: "column" },
    {
      key: "reject_reason",
      header: "Reason",
      type: "accessor",
      accessor: "reject_reason",
    },
  ];

export function parseEvent(
  content: ConversationMessageContentDto,
): AgentEvent | null {
  try {
    return JSON.parse(content.content) as AgentEvent;
  } catch {
    return null;
  }
}

export function isCompletedEvent(event: AgentEvent): boolean {
  return !IN_PROGRESS_EVENTS.has(event.event_name);
}

export function hasContent(event: AgentEvent): boolean {
  if (
    event.event_name === AgentEventName.Starting ||
    event.event_name === AgentEventName.Completed
  ) {
    return false;
  }
  if (event.event_name === AgentEventName.ExecutedFunctionalRoleAnalysis) {
    const fra = event.data as ExecutedFunctionalRoleAnalysisData | null;
    return Boolean(fra?.functional_role_analysis_result);
  }
  return true;
}

function HypothesisContent({
  data,
}: {
  data: GeneratedCandidateHypothesisData;
}) {
  return (
    <Flex direction="column" gap="xs">
      {data.map((hypothesis, i) => (
        <Fragment key={i}>
          <Text fontSize="xs">candidate plan {i + 1}</Text>
          <Box
            className={styles.hypothesisItem}
            backgroundColor="grey"
            borderRadius="md"
            paddingX="sm"
            paddingY="xs"
          >
            {hypothesis}
          </Box>
        </Fragment>
      ))}
    </Flex>
  );
}

function MasterPlanContent({ data }: { data: IntegratedCandidatePlansData }) {
  return (
    <Box
      className={styles.masterPlan}
      backgroundColor="grey"
      borderRadius="md"
      paddingX="sm"
      paddingY="xs"
    >
      {data.master_plan}
    </Box>
  );
}

function PrunedCatalogContent({ data }: { data: DataCatalogDatabase }) {
  const schemas = Object.values(data.schemas);
  if (schemas.length === 0) {
    return (
      <Text fontSize="xs" color="subtle">
        No schemas in pruned catalog
      </Text>
    );
  }
  return (
    <Flex direction="column" gap="2xs">
      {schemas.map((schema) => {
        const tables = Object.values(schema.tables);
        return (
          <Box key={schema.name}>
            <Text fontSize="xs">
              <span className={styles.catalogSchemaName}>{schema.name}</span>
            </Text>
            <Flex gap="2xs" sx={{ flexWrap: "wrap" }}>
              {tables.map((table) => (
                <span key={table.name} className={styles.catalogTableChip}>
                  {table.name}
                </span>
              ))}
            </Flex>
          </Box>
        );
      })}
    </Flex>
  );
}

function SearchSpaceReductionContent({
  data,
}: {
  data: ExecutedSearchSpaceReductionData;
}) {
  return <PrunedCatalogContent data={data.pruned_data_catalog} />;
}

function FunctionalRoleAnalysisContent({
  data,
}: {
  data: ExecutedFunctionalRoleAnalysisData;
}) {
  const r = data.functional_role_analysis_result;
  return (
    <Flex direction="column" gap="xs">
      <Box>
        <Text fontSize="xs" color="subtle">
          Database structure
        </Text>
        <Box
          className={styles.masterPlan}
          backgroundColor="grey"
          borderRadius="md"
          paddingX="sm"
          paddingY="xs"
          marginTop="2xs"
        >
          {r.database_structure}
        </Box>
      </Box>
      <Box>
        <Text fontSize="xs" color="subtle">
          Query-specific content analysis
        </Text>
        <Box
          className={styles.masterPlan}
          backgroundColor="grey"
          borderRadius="md"
          paddingX="sm"
          paddingY="xs"
          marginTop="2xs"
        >
          {r.query_specific_content_analysis}
        </Box>
      </Box>
      {r.table_functions.length > 0 && (
        <Box>
          <Text fontSize="xs" color="subtle">
            Table functions
          </Text>
          <Flex direction="column" gap="2xs" marginTop="2xs">
            {r.table_functions.map((tf, i) => (
              <Box
                key={`${tf.schema}.${tf.table}-${i}`}
                backgroundColor="grey"
                borderRadius="md"
                paddingX="sm"
                paddingY="xs"
              >
                <Text fontSize="xs">
                  <span className={styles.refinedTableName}>
                    {tf.schema}.{tf.table}
                  </span>
                </Text>
                <Box className={styles.observationSummary}>
                  {tf.table_function}
                </Box>
              </Box>
            ))}
          </Flex>
        </Box>
      )}
    </Flex>
  );
}

function DataProfilingContent({ data }: { data: ExecutedDataProfilingData }) {
  const observations = data.emperical_observations;
  if (observations.length === 0) {
    return (
      <Text fontSize="xs" color="subtle">
        No observations
      </Text>
    );
  }
  return (
    <Flex direction="column" gap="xs">
      {observations.map((obs, i) => (
        <ObservationCard key={i} observation={obs} />
      ))}
    </Flex>
  );
}

function ObservationCard({
  observation,
}: {
  observation: EmpericalObservation;
}) {
  return (
    <Box backgroundColor="grey" borderRadius="md" paddingX="sm" paddingY="xs">
      <Flex gap="2xs" alignItems="center">
        <Text fontSize="xs" color={observation.relevant ? "success" : "subtle"}>
          {observation.relevant ? "Relevant" : "Not relevant"}
        </Text>
      </Flex>
      {observation.table_summary && (
        <Box className={styles.observationSummary}>
          {observation.table_summary}
        </Box>
      )}
      {observation.relevant_columns.length > 0 && (
        <SophoTable
          size="compact"
          columns={OBSERVATION_COLUMNS}
          data={observation.relevant_columns}
        />
      )}
    </Box>
  );
}

function SchemaLinkingContent({
  data,
}: {
  data: SchemaLinkingFinalSynthesisResponse;
}) {
  return (
    <Flex direction="column" gap="xs">
      {data.refined_schema.map((table) => (
        <Box
          key={table.table_name}
          backgroundColor="grey"
          borderRadius="md"
          paddingX="sm"
          paddingY="xs"
        >
          <Text fontSize="xs">
            <span className={styles.refinedTableName}>{table.table_name}</span>
          </Text>
          <Flex direction="column" gap="2xs" paddingLeft="sm">
            {table.relevant_columns.map((col) => (
              <Box key={col.column_name} className={styles.refinedColumn}>
                <span className={styles.refinedColumnName}>
                  {col.column_name}
                </span>
                {col.relevance_reason && (
                  <Text fontSize="xs" color="subtle">
                    {" "}
                    — {col.relevance_reason}
                  </Text>
                )}
              </Box>
            ))}
          </Flex>
        </Box>
      ))}
      {data.rejected_candidates.length > 0 && (
        <Box marginTop="xs">
          <Text fontSize="xs" color="subtle">
            <span className={styles.rejectedLabel}>Rejected Candidates</span>
          </Text>
          <SophoTable
            size="compact"
            columns={REJECTED_CANDIDATE_COLUMNS}
            data={data.rejected_candidates}
          />
        </Box>
      )}
    </Flex>
  );
}

function SqlContent({ data }: { data: GeneratedSqlData }) {
  return (
    <Box
      className={styles.sqlBlock}
      backgroundColor="grey"
      borderRadius="md"
      paddingX="sm"
      paddingY="xs"
    >
      {data.sql}
    </Box>
  );
}

function ErrorContent({ data }: { data: string }) {
  return <Box className={styles.errorBlock}>{data}</Box>;
}

export type ExtractedRouterDecision = {
  data: RouterDecisionData;
  createdAt?: string;
};

export function extractRouterDecision(
  events: AgentEvent[],
  contents?: ConversationMessageContentDto[],
): ExtractedRouterDecision | null {
  const routed = events.find((e) => e.event_name === AgentEventName.Routed);
  if (!routed?.data) return null;
  const d = routed.data as RouterDecisionData;
  if (
    d.decision.code === RouterCode.TextToSql ||
    d.decision.code === RouterCode.Followup
  ) {
    return null;
  }

  let createdAt: string | undefined;
  if (contents) {
    for (const content of contents) {
      const event = parseEvent(content);
      if (event?.event_name === AgentEventName.Routed) {
        createdAt = content.created_at;
        break;
      }
    }
  }

  return { data: d, createdAt };
}

const ROUTER_DECISION_BADGE: Record<
  Exclude<RouterCode, typeof RouterCode.TextToSql | typeof RouterCode.Followup>,
  { label: string; variant: NonNullable<BadgeProps["variant"]> }
> = {
  [RouterCode.Clarify]: { label: "Clarification", variant: "yellow" },
  [RouterCode.RejectOffTopic]: {
    label: "Can't help with this",
    variant: "subtle",
  },
  [RouterCode.RejectUnsafe]: {
    label: "Can't help with this",
    variant: "default",
  },
};

export function RouterDecisionBubble({
  data,
  createdAt,
}: {
  data: RouterDecisionData;
  createdAt?: string;
}) {
  const { code, message } = data.decision;
  if (code === RouterCode.TextToSql || code === RouterCode.Followup)
    return null;

  const { label, variant } = ROUTER_DECISION_BADGE[code];

  return (
    <Flex direction="column" className={panelStyles.messageBubbleGroup}>
      <Box backgroundColor="grey" borderRadius="md" paddingX="sm" paddingY="xs">
        <Badge variant={variant}>{label}</Badge>
        {message && (
          <Box className={styles.routerDecisionMessage} marginTop="2xs">
            <Text as="div" fontSize="base">
              {message}
            </Text>
          </Box>
        )}
      </Box>
      <MessageHoverFooter
        createdAt={createdAt ?? ""}
        textToCopy={message}
        footerClassName={panelStyles.messageFooter}
      />
    </Flex>
  );
}

export function renderStepContent(event: AgentEvent): React.ReactNode | null {
  switch (event.event_name) {
    case AgentEventName.GeneratedCandidateHypothesis:
      return (
        <HypothesisContent
          data={event.data as GeneratedCandidateHypothesisData}
        />
      );
    case AgentEventName.IntegratedCandidatePlans:
      return (
        <MasterPlanContent data={event.data as IntegratedCandidatePlansData} />
      );
    case AgentEventName.ExecutedSearchSpaceReduction:
      return (
        <SearchSpaceReductionContent
          data={event.data as ExecutedSearchSpaceReductionData}
        />
      );
    case AgentEventName.ExecutedFunctionalRoleAnalysis:
      return (
        <FunctionalRoleAnalysisContent
          data={event.data as ExecutedFunctionalRoleAnalysisData}
        />
      );
    case AgentEventName.ExecutedDataProfiling:
      return (
        <DataProfilingContent data={event.data as ExecutedDataProfilingData} />
      );
    case AgentEventName.ExecutedSchemaLinkingSynthesis:
      return (
        <SchemaLinkingContent
          data={
            (event.data as ExecutedSchemaLinkingSynthesisData).linked_schema
          }
        />
      );
    case AgentEventName.GeneratedSql:
      return <SqlContent data={event.data as GeneratedSqlData} />;
    case AgentEventName.Error:
      return <ErrorContent data={event.data as string} />;
    default:
      return null;
  }
}

type TraceStepProps = {
  event: AgentEvent;
  isLast: boolean;
  isActive?: boolean;
  defaultOpen?: boolean;
};

export function TraceStep({
  event,
  isLast,
  isActive = false,
  defaultOpen = false,
}: TraceStepProps) {
  const [open, setOpen] = useState(defaultOpen);
  const label = STEP_LABELS[event.event_name] ?? event.event_name;
  const isError = event.event_name === AgentEventName.Error;
  const stepIconColor: IconColor = isError ? "red" : TRACE_STEP_ICON_COLOR;
  const expandable = !isActive && hasContent(event);

  return (
    <Collapsible
      open={open}
      onOpenChange={expandable ? setOpen : undefined}
      disabled={!expandable}
    >
      <Flex
        direction="column"
        className={`${styles.step} ${!expandable ? styles.noContentStep : ""}`}
      >
        <Flex gap="2xs" alignItems="center">
          <Flex
            alignItems="center"
            justifyContent="center"
            className={styles.stepIconWrapper}
          >
            {isActive ? (
              <Spinner size="sm" color="grey" />
            ) : (
              <Icon
                type={TRACE_STEP_ICON}
                color={stepIconColor}
                size="sm"
                interactive={false}
              />
            )}
          </Flex>
          <Box flex="grow">
            <Collapsible.Trigger>
              <Flex alignItems="center" gap="2xs" className={styles.stepHeader}>
                <MotionFlex
                  className={styles.stepHeaderChevron}
                  alignItems="center"
                  justifyContent="center"
                  animate={{ rotate: open ? 90 : 0 }}
                  transition={rotateToggleTransition}
                >
                  <Icon
                    type="chevron_right"
                    color="grey"
                    size="sm"
                    interactive={false}
                  />
                </MotionFlex>
                <Text fontSize="xs" color="darkGrey">
                  {label}
                </Text>
              </Flex>
            </Collapsible.Trigger>
          </Box>
        </Flex>
        <Flex gap="2xs" sx={{ minHeight: !isLast ? "16px" : undefined }}>
          <Flex direction="column" alignItems="center" className={styles.stepLineColumn}>
            {!isLast && <Box className={styles.stepLine} />}
          </Flex>
          {expandable && (
            <Box flex="grow" paddingBottom="sm">
              <Collapsible.Content>
                <Box paddingTop="2xs" paddingLeft="md">
                  {renderStepContent(event)}
                </Box>
              </Collapsible.Content>
            </Box>
          )}
        </Flex>
      </Flex>
    </Collapsible>
  );
}
