import { ChartType } from "src/components/Chart";
import { ExecuteCellResponseDto } from "src/components/Notebook/Cell/dto";

export type MessageSegment =
  | { type: "TEXT"; text: string }
  | { type: "COMMAND"; name: string };

export type CreateConversationDto = {
  connection_id: string;
  segments: MessageSegment[];
};

export type AppendUserMessageDto = {
  segments: MessageSegment[];
};

export type ConversationDto = {
  id: string;
  connection_id: string;
  name: string;
  status: string;
  created_at: string;
  updated_at: string;
};

export type ConversationListItemDto = ConversationDto & {
  user_message_count: number;
};

export type PaginatedConversationsDto = {
  items: ConversationListItemDto[];
  total: number;
  page: number;
  page_size: number;
};

export type ListConversationsParams = {
  page?: number;
  pageSize?: number;
  search?: string;
};

export type BulkDeleteConversationsDto = {
  conversation_ids: string[];
};

export type ConversationMessageContentDto = {
  id: string;
  conversation_message_id: string;
  sequence_number: number;
  content_type: string;
  content: string;
  status: string;
  created_at: string;
  updated_at: string;
};

export const Sender = {
  Human: "HUMAN",
  Assistant: "ASSISTANT",
} as const;

export type Sender = (typeof Sender)[keyof typeof Sender];

export const MessageStatus = {
  Processing: "PROCESSING",
  Processed: "PROCESSED",
  Failed: "FAILED",
  AwaitingClarification: "AWAITING_CLARIFICATION",
  Rejected: "REJECTED",
} as const;

export type MessageStatus = (typeof MessageStatus)[keyof typeof MessageStatus];

export type ConversationMessageDto = {
  id: string;
  conversation_id: string;
  sequence_number: number;
  sender: Sender;
  status: MessageStatus;
  created_at: string;
  updated_at: string;
  content: ConversationMessageContentDto[];
};

export type ConversationWithMessagesDto = {
  conversation: ConversationDto;
  messages: ConversationMessageDto[];
  should_execute_completion: boolean;
};

export type SchemaLinkingRelevantColumn = {
  column_name: string;
  relevance_reason: string;
};

export type SchemaLinkingRefinedTable = {
  table_name: string;
  relevant_columns: SchemaLinkingRelevantColumn[];
};

export type SchemaLinkingRejectedCandidate = {
  table: string;
  column: string;
  reject_reason: string;
};

export type SchemaLinkingFinalSynthesisResponse = {
  refined_schema: SchemaLinkingRefinedTable[];
  rejected_candidates: SchemaLinkingRejectedCandidate[];
  exploration_queries: string[];
  status: string;
};

export type EmpericalObservationColumn = {
  column_name: string;
  relevance_reason: string;
  observations: string;
};

export type EmpericalObservation = {
  relevant: boolean;
  relevant_columns: EmpericalObservationColumn[];
  table_summary: string;
};

export type DataCatalogColumn = {
  name: string;
  data_type: string;
  description: string;
  sample_values: string[];
};

export type DataCatalogTable = {
  name: string;
  description: string;
  columns: Record<string, DataCatalogColumn>;
};

export type DataCatalogSchema = {
  name: string;
  description: string;
  tables: Record<string, DataCatalogTable>;
};

export type DataCatalogDatabase = {
  name: string;
  description: string;
  schemas: Record<string, DataCatalogSchema>;
};

export const AgentEventName = {
  Starting: "starting",
  Routing: "routing",
  Routed: "routed",
  Error: "error",
  Completed: "completed",
  AwaitingClarification: "awaiting_clarification",
  Rejected: "rejected",
  GeneratingCandidateHypothesis: "generating_candidate_hypothesis",
  GeneratedCandidateHypothesis: "generated_candidate_hypothesis",
  IntegratingCandidatePlans: "integrating_candidate_plans",
  IntegratedCandidatePlans: "integrated_candidate_plans",
  ExecutingSearchSpaceReduction: "executing_search_space_reduction",
  ExecutedSearchSpaceReduction: "executed_search_space_reduction",
  ExecutingFunctionalRoleAnalysis: "executing_functional_role_analysis",
  ExecutedFunctionalRoleAnalysis: "executed_functional_role_analysis",
  ExecutingDataProfiling: "executing_data_profiling",
  ExecutedDataProfiling: "executed_data_profiling",
  ExecutingSchemaLinkingSynthesis: "executing_schema_linking_synthesis",
  ExecutedSchemaLinkingSynthesis: "executed_schema_linking_synthesis",
  GeneratingSql: "generating_sql",
  GeneratedSql: "generated_sql",
  ExecutingQuery: "executing_query",
  ExecutedQuery: "executed_query",
  RecommendingVisualization: "recommending_visualization",
  RecommendedVisualization: "recommended_visualization",
  SuggestedFollowups: "suggested_followups",
  GeneratingCanvas: "generating_canvas",
  CanvasGenerated: "canvas_generated",
} as const;

export type AgentEventName =
  (typeof AgentEventName)[keyof typeof AgentEventName];

export const IN_PROGRESS_EVENTS = new Set<string>([
  AgentEventName.Routing,
  AgentEventName.GeneratingCandidateHypothesis,
  AgentEventName.IntegratingCandidatePlans,
  AgentEventName.ExecutingSearchSpaceReduction,
  AgentEventName.ExecutingFunctionalRoleAnalysis,
  AgentEventName.ExecutingDataProfiling,
  AgentEventName.ExecutingSchemaLinkingSynthesis,
  AgentEventName.GeneratingSql,
  AgentEventName.ExecutingQuery,
  AgentEventName.RecommendingVisualization,
  AgentEventName.GeneratingCanvas,
]);

export type AgentEvent = {
  event_name: AgentEventName;
  data?: unknown;
};

export type GeneratedCandidateHypothesisData = string[];

export type IntegratedCandidatePlansData = {
  master_plan: string;
};

export type ExecutedSearchSpaceReductionData = {
  pruned_data_catalog: DataCatalogDatabase;
};

export type TableFunction = {
  database: string;
  schema: string;
  table: string;
  table_function: string;
};

export type FunctionalRoleAnalysisResult = {
  database_structure: string;
  query_specific_content_analysis: string;
  table_functions: TableFunction[];
};

export type ExecutedFunctionalRoleAnalysisData = {
  functional_role_analysis_result: FunctionalRoleAnalysisResult;
};

export type ExecutedDataProfilingData = {
  emperical_observations: EmpericalObservation[];
};

export type ExecutedSchemaLinkingSynthesisData = {
  linked_schema: SchemaLinkingFinalSynthesisResponse;
};

export type GeneratedSqlData = {
  sql: string;
};

export const RouterCode = {
  TextToSql: "text_to_sql",
  Followup: "followup",
  Clarify: "clarify",
  RejectOffTopic: "reject_off_topic",
  RejectUnsafe: "reject_unsafe",
  GenerateCanvas: "generate_canvas",
} as const;

export type RouterCode = (typeof RouterCode)[keyof typeof RouterCode];

export type RouterDecision = {
  code: RouterCode;
  message: string;
};

export type RouterDecisionData = {
  decision: RouterDecision;
};

export type RecommendedVisualizationData = {
  visualization: {
    chart_type: ChartType;
    x_axis?: string;
    y_axis?: string;
    category?: string;
    value?: string;
    reasoning: string;
  };
};

export type SuggestedFollowupsData = {
  questions: string[];
};

export type CanvasGeneratedData = {
  canvas_id: string;
  name: string;
  description: string | null;
  sql_cell_count: number;
  chart_cell_count: number;
  dashboard_charts_count: number;
};

export const ArtifactType = {
  Canvas: "CANVAS",
} as const;

export type ArtifactType = (typeof ArtifactType)[keyof typeof ArtifactType];

export type Artifact = {
  id: string;
  type: typeof ArtifactType.Canvas;
  data: CanvasGeneratedData;
};

export type ChartRenderData = {
  queryData: ExecuteCellResponseDto;
  visualization: RecommendedVisualizationData["visualization"];
};

export function extractFollowUpQuestions(
  messages: ConversationMessageDto[],
): string[] {
  for (let i = messages.length - 1; i >= 0; i--) {
    const message = messages[i];
    if (message.sender !== Sender.Assistant) continue;
    for (const content of message.content) {
      try {
        const event = JSON.parse(content.content) as AgentEvent;
        if (event.event_name === AgentEventName.SuggestedFollowups) {
          const data = event.data as { questions?: string[] } | undefined;
          return data?.questions ?? [];
        }
      } catch {
        /* ignore unparseable content */
      }
    }
    return [];
  }
  return [];
}

export function extractChartData(events: AgentEvent[]): ChartRenderData | null {
  const queryEvent = events.find(
    (e) => e.event_name === AgentEventName.ExecutedQuery,
  );
  const vizEvent = events.find(
    (e) => e.event_name === AgentEventName.RecommendedVisualization,
  );

  if (!queryEvent?.data || !vizEvent?.data) return null;

  const queryData = queryEvent.data as ExecuteCellResponseDto;
  const vizData = vizEvent.data as RecommendedVisualizationData;

  return {
    queryData,
    visualization: vizData.visualization,
  };
}

export function extractCanvasGenerated(
  events: AgentEvent[],
): CanvasGeneratedData | null {
  const event = events.find(
    (e) => e.event_name === AgentEventName.CanvasGenerated,
  );
  return event?.data ? (event.data as CanvasGeneratedData) : null;
}

function toCanvasArtifact(
  data: CanvasGeneratedData | undefined,
): Artifact | null {
  if (!data?.canvas_id) return null;
  return { id: data.canvas_id, type: ArtifactType.Canvas, data };
}

function toArtifact(event: AgentEvent): Artifact | null {
  if (event.event_name !== AgentEventName.CanvasGenerated) return null;
  return toCanvasArtifact(event.data as CanvasGeneratedData | undefined);
}

export function extractArtifactsFromEvents(events: AgentEvent[]): Artifact[] {
  return events
    .map(toArtifact)
    .filter((artifact): artifact is Artifact => artifact !== null);
}

export function extractArtifactsFromMessages(
  messages: ConversationMessageDto[],
): Artifact[] {
  const artifacts: Artifact[] = [];

  for (const message of messages) {
    if (message.sender !== Sender.Assistant) continue;
    for (const content of message.content) {
      try {
        const artifact = toArtifact(JSON.parse(content.content) as AgentEvent);
        if (artifact) artifacts.push(artifact);
      } catch {
        /* ignore unparseable content */
      }
    }
  }

  return artifacts;
}
