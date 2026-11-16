import { ChartSeriesSpec } from "src/components/Chart/ChartCommon";
import { SegmentedControlOption } from "src/components/design-system";
import { APP_ROUTES } from "src/constants/app_routes";

export const RANGE_SEARCH_PARAMETER = "days";
export const KEY_SEARCH_PARAMETER = "key";
export const ALL_KEYS_OPTION = "ALL";
export const KEY_OPTIONS = [
  { value: ALL_KEYS_OPTION, label: "All usage" },
  { value: "TENANT", label: "Own key" },
  { value: "PLATFORM", label: "Platform AI" },
];
export const DEFAULT_RANGE_OPTION = "30";
export const RANGE_OPTIONS = [
  { value: "7", label: "Last 7 days" },
  { value: "30", label: "Last 30 days" },
  { value: "90", label: "Last 90 days" },
];

export const USAGE_TABS: (SegmentedControlOption & { label: string })[] = [
  { value: APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_SUMMARY, label: "Summary" },
  { value: APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_USER, label: "By User" },
  { value: APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_WORK, label: "By Work" },
  {
    value: APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_CONVERSATION,
    label: "By Conversation",
  },
];

export const CONVERSATION_PAGE_SIZE = 10;
export const CONVERSATION_PAGE_SEARCH_PARAMETER = "page";
export const CONVERSATION_ID_PATH_PARAMETER = ":conversationId";

export const GRID_GUTTER = { base: "md", md: "lg" } as const;
export const TILE_COLUMN_SPAN = { base: 1, md: 3 } as const;
export const QUESTION_TILE_COLUMN_SPAN = { base: 1, md: 4 } as const;
export const CHART_HEIGHT = "20rem";
export const PIE_CHART_HEIGHT = "28rem";

export const USERS_TABLE_COLUMNS = "minmax(0, 1fr) 7rem 5rem";
export const CONVERSATIONS_TABLE_COLUMNS =
  "minmax(0, 1.5fr) minmax(0, 1fr) 7rem 5rem";
export const QUESTIONS_TABLE_COLUMNS = "minmax(0, 1fr) 7rem 5rem";

export const TOKENS_PER_DAY_SERIES: ChartSeriesSpec[] = [
  { dataKey: "input_tokens", name: "Input tokens", colorIndex: 0 },
  { dataKey: "output_tokens", name: "Output tokens", colorIndex: 1 },
];
export const TOKENS_PER_DAY_X_AXIS = "day";
export const KIND_CHART_CATEGORY = "label";
export const KIND_CHART_VALUE = "total_tokens";

export const PAGE_DESCRIPTION = "AI usage reports";
export const EMPTY_HEADING = "No AI usage in this range";
export const EMPTY_DESCRIPTION =
  "When users ask questions, generate summaries or build canvases, the tokens each one uses appear here, by day, by user and by conversation.";
