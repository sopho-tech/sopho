import { ConversationNameKind } from "src/api/ai_usage";

export type PlatformUsageTenantDto = {
  tenant_id: string;
  tenant_name: string;
  total_tokens: number;
  call_count: number;
  platform_ai_used_tokens: number;
  platform_ai_token_allowance: number | null;
};

export type PlatformUsageUserDto = {
  user_id: string | null;
  name: string;
  tenant_id: string;
  tenant_name: string;
  total_tokens: number;
  call_count: number;
};

export type PlatformUsageConversationDto = {
  conversation_id: string;
  tenant_id: string;
  tenant_name: string;
  name: string;
  name_kind: ConversationNameKind;
  user_name: string;
  total_tokens: number;
  call_count: number;
};

export type PlatformUsagePage<UsageRow> = {
  data: UsageRow[];
  total_pages: number;
  page: number;
  page_size: number;
  total_items: number;
};
