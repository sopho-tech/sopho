export type UsageRangeParams = {
  lastDayCount: string;
  keySource: string | null;
};

export type UsageSummaryDto = {
  from: string;
  to: string;
  input_tokens: number;
  output_tokens: number;
  cached_input_tokens: number;
  total_tokens: number;
  call_count: number;
  has_usage: boolean;
  is_platform_ai_on: boolean;
  missing_token_counts_note: string | null;
};

export type UsageDayDto = {
  day: string;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  call_count: number;
};

export type UsageOverTimeDto = {
  days: UsageDayDto[];
};

export type UsageUserDto = {
  user_id: string | null;
  name: string;
  total_tokens: number;
  call_count: number;
};

export type UsageByUserDto = {
  users: UsageUserDto[];
};

export type UsageKindDto = {
  kind: string;
  label: string;
  total_tokens: number;
  call_count: number;
};

export type UsageByKindDto = {
  kinds: UsageKindDto[];
};

export type ConversationNameKind = "NAME" | "PRIVATE" | "DELETED";

export type UsageConversationDto = {
  conversation_id: string;
  name: string;
  name_kind: ConversationNameKind;
  user_name: string;
  total_tokens: number;
  call_count: number;
};

export type UsageByConversationDto = {
  data: UsageConversationDto[];
  total_pages: number;
  page: number;
  page_size: number;
  total_items: number;
};

export type QuestionTextKind =
  "TEXT" | "COMMAND" | "PRIVATE" | "DELETED" | "NO_QUESTION";

export type UsageQuestionDto = {
  conversation_message_id: string | null;
  text: string;
  text_kind: QuestionTextKind;
  first_call_at: string;
  total_tokens: number;
  call_count: number;
};

export type UsageConversationQuestionsDto = {
  conversation_id: string;
  name: string;
  name_kind: ConversationNameKind;
  total_tokens: number;
  call_count: number;
  question_count: number;
  questions: UsageQuestionDto[];
};
