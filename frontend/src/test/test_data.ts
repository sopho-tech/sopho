import type { UserDto } from "src/api/auth_api";
import type { CanvasDto } from "src/components/Canvases/dto";
import { StatusType, type ConnectionDto } from "src/components/Connection/dto";
import type { DashboardDto } from "src/components/Dashboard/dto";
import {
  CellType,
  type CellDto,
  type ExecuteCellResponseDto,
} from "src/components/Notebook/Cell/dto";
import type { NotebookDto } from "src/components/Notebook/dto";
import {
  AgentEventName,
  MessageStatus,
  Sender,
  type AgentEvent,
  type ConversationDto,
  type ConversationMessageContentDto,
  type ConversationMessageDto,
  type ConversationWithMessagesDto,
} from "src/components/ConversationalAnalytics/dto";
import { ColumnDataType } from "src/constants/database_types";

export const TEST_USER: UserDto = {
  id: "00000000-0000-4000-8000-000000000001",
  username: "admin",
  email: "admin@sopho.test",
  full_name: "Test Admin",
};

export const SQLITE_CONNECTION: ConnectionDto = {
  id: "00000000-0000-4000-8000-000000000101",
  name: "Ecommerce",
  source_type: "SQLITE",
  host: "",
  port: null,
  database: "/data/ecommerce.db",
  schema: null,
  username: "",
  password: "",
  description: null,
  status: StatusType.Active,
  created_at: "2026-10-01T09:00:00Z",
  updated_at: "2026-10-01T09:00:00Z",
};

export const POSTGRES_CONNECTION: ConnectionDto = {
  id: "00000000-0000-4000-8000-000000000102",
  name: "Warehouse",
  source_type: "POSTGRESQL",
  host: "db.internal",
  port: 5432,
  database: "warehouse",
  schema: "public",
  username: "analyst",
  password: "analyst-password",
  description: "Reporting replica",
  status: StatusType.Active,
  created_at: "2026-10-01T09:00:00Z",
  updated_at: "2026-10-01T09:00:00Z",
};

export const SSE_TEST_STREAM_URL =
  "http://localhost/api/v1/conversational_analytics/stream";

export const CANVAS_ID = "00000000-0000-4000-8000-000000000201";
export const NOTEBOOK_ID = "00000000-0000-4000-8000-000000000301";
export const DASHBOARD_ID = "00000000-0000-4000-8000-000000000501";

export const CANVAS: CanvasDto = {
  id: CANVAS_ID,
  name: "Sales overview",
  description: "Orders and their statuses",
  status: "ACTIVE",
  created_at: "2026-10-01T09:00:00Z",
  updated_at: "2026-10-01T09:00:00Z",
  sql_cell_count: 1,
  chart_cell_count: 1,
  dashboard_charts_count: 1,
};

export const ORDER_STATUS_SQL =
  "SELECT order_status, COUNT(*) AS order_count FROM orders GROUP BY order_status";
export const ORDER_COUNT_SQL = "SELECT COUNT(*) AS order_count FROM orders";

export const SQL_CELL: CellDto = {
  id: "00000000-0000-4000-8000-000000000401",
  notebook_id: NOTEBOOK_ID,
  connection_id: SQLITE_CONNECTION.id,
  display_order: 1,
  name: "Orders by status",
  content: `${ORDER_STATUS_SQL};\n${ORDER_COUNT_SQL};`,
  cell_type: CellType.SQL,
  status: "ACTIVE",
};

export const CHART_CELL: CellDto = {
  id: "00000000-0000-4000-8000-000000000402",
  notebook_id: NOTEBOOK_ID,
  connection_id: SQLITE_CONNECTION.id,
  display_order: 2,
  name: "Orders by status chart",
  content: JSON.stringify({
    chart_type: "BAR",
    cell_id: SQL_CELL.id,
    x_axis: "order_status",
    x_axis_alias: "order_status",
    series: [
      {
        column: "order_count",
        aggregate_function: "SUM",
        label: "Orders",
        alias: "order_count_sum",
        color_index: 0,
      },
      {
        column: "order_count",
        aggregate_function: "MAX",
        label: "Largest status",
        alias: "order_count_max",
        color_index: 1,
      },
    ],
    y_axis_sort_by: "order_count_sum",
    y_axis_sort_order: "DESC",
  }),
  cell_type: CellType.CHART,
  status: "ACTIVE",
};

export const ORDER_STATUS_RESULT: ExecuteCellResponseDto = {
  columns: [
    { column_name: "order_status", data_type: ColumnDataType.TEXT },
    { column_name: "order_count", data_type: ColumnDataType.INT8 },
  ],
  data: [
    { order_status: "delivered", order_count: 9620 },
    { order_status: "shipped", order_count: 118 },
    { order_status: "canceled", order_count: 75 },
  ],
};

export const ORDER_COUNT_RESULT: ExecuteCellResponseDto = {
  columns: [{ column_name: "order_count", data_type: ColumnDataType.INT8 }],
  data: [{ order_count: 10000 }],
};

export const ORDER_STATUS_CHART_RESULT: ExecuteCellResponseDto = {
  columns: [
    { column_name: "order_status", data_type: ColumnDataType.TEXT },
    { column_name: "order_count_sum", data_type: ColumnDataType.INT8 },
    { column_name: "order_count_max", data_type: ColumnDataType.INT8 },
  ],
  data: [
    { order_status: "delivered", order_count_sum: 9620, order_count_max: 9620 },
    { order_status: "shipped", order_count_sum: 118, order_count_max: 118 },
    { order_status: "canceled", order_count_sum: 75, order_count_max: 75 },
  ],
};

export function dashboardWithCharts(chartCells: CellDto[]): DashboardDto {
  return {
    id: DASHBOARD_ID,
    name: "Sales overview",
    description: "Orders and their statuses",
    status: "ACTIVE",
    layout: chartCells.map((chartCell, position) => ({
      cell_id: chartCell.id,
      notebook_id: NOTEBOOK_ID,
      x_position: 0,
      y_position: position * 4,
      x_size: 6,
      y_size: 4,
    })),
  };
}

export function notebookWithCells(cells: CellDto[]): NotebookDto {
  return {
    id: NOTEBOOK_ID,
    name: "Sales notebook",
    description: null,
    status: "ACTIVE",
    cells,
    created_at: "2026-10-01T09:00:00Z",
    updated_at: "2026-10-01T09:00:00Z",
  };
}

export const CONVERSATION_ID = "00000000-0000-4000-8000-000000000601";
export const ORDER_STATUS_QUESTION = "How many orders are in each status?";
export const ORDER_STATUS_NARRATION =
  "Delivered orders far outnumber every other status.";

export const CONVERSATION: ConversationDto = {
  id: CONVERSATION_ID,
  connection_id: SQLITE_CONNECTION.id,
  name: "Untitled",
  status: "ACTIVE",
  created_at: "2026-10-15T10:00:00Z",
  updated_at: "2026-10-15T10:00:00Z",
};

export const ORDER_STATUS_ANSWER_EVENTS: AgentEvent[] = [
  { event_name: AgentEventName.Starting },
  { event_name: AgentEventName.Routing },
  {
    event_name: AgentEventName.Routed,
    data: { decision: { code: "text_to_sql", message: ORDER_STATUS_QUESTION } },
  },
  { event_name: AgentEventName.GeneratingSql },
  { event_name: AgentEventName.GeneratedSql, data: { sql: ORDER_STATUS_SQL } },
  { event_name: AgentEventName.ExecutingQuery },
  { event_name: AgentEventName.ExecutedQuery, data: ORDER_STATUS_RESULT },
  { event_name: AgentEventName.Narrating },
  {
    event_name: AgentEventName.Narrated,
    data: { narration: ORDER_STATUS_NARRATION },
  },
  { event_name: AgentEventName.RecommendingVisualization },
  {
    event_name: AgentEventName.RecommendedVisualization,
    data: {
      visualization: {
        chart_type: "BAR",
        x_axis: "order_status",
        series: [
          { data_key: "order_count", name: "order_count", color_index: 0 },
        ],
      },
    },
  },
];

function messageContent(
  messageId: string,
  sequenceNumber: number,
  contentType: string,
  content: string,
): ConversationMessageContentDto {
  return {
    id: `${messageId}-${sequenceNumber}`,
    conversation_message_id: messageId,
    sequence_number: sequenceNumber,
    content_type: contentType,
    content,
    status: "ACTIVE",
    created_at: "2026-10-15T10:00:00Z",
    updated_at: "2026-10-15T10:00:00Z",
  };
}

export function conversationWithMessages(
  assistantEvents: AgentEvent[] | null,
): ConversationWithMessagesDto {
  const questionMessageId = "00000000-0000-4000-8000-000000000701";
  const answerMessageId = "00000000-0000-4000-8000-000000000702";
  const questionMessage: ConversationMessageDto = {
    id: questionMessageId,
    conversation_id: CONVERSATION_ID,
    sequence_number: 1,
    sender: Sender.Human,
    status: MessageStatus.Processed,
    created_at: "2026-10-15T10:00:00Z",
    updated_at: "2026-10-15T10:00:00Z",
    content: [
      messageContent(
        questionMessageId,
        1,
        "TEXT",
        JSON.stringify([{ type: "TEXT", text: ORDER_STATUS_QUESTION }]),
      ),
    ],
  };
  const answerMessages: ConversationMessageDto[] =
    assistantEvents === null
      ? []
      : [
          {
            id: answerMessageId,
            conversation_id: CONVERSATION_ID,
            sequence_number: 2,
            sender: Sender.Assistant,
            status: MessageStatus.Processed,
            created_at: "2026-10-15T10:00:05Z",
            updated_at: "2026-10-15T10:00:05Z",
            content: assistantEvents.map((event, index) =>
              messageContent(
                answerMessageId,
                index + 1,
                event.event_name === AgentEventName.Narrated
                  ? "NARRATION"
                  : "DATA_ANALYSIS_RESPONSE",
                JSON.stringify(event),
              ),
            ),
          },
        ];
  return {
    conversation: CONVERSATION,
    messages: [questionMessage, ...answerMessages],
    should_execute_completion: assistantEvents === null,
    user_message_limit_reached: false,
  };
}
