import { SegmentedControlOption } from "src/components/design-system";
import { APP_ROUTES } from "src/constants/app_routes";

export const PLATFORM_USAGE_TABS: (SegmentedControlOption & {
  label: string;
})[] = [
  {
    value: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_SUMMARY,
    label: "Summary",
  },
  {
    value: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_TENANT,
    label: "By Tenant",
  },
  {
    value: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_USER,
    label: "By User",
  },
  {
    value: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_WORK,
    label: "By Work",
  },
  {
    value: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_CONVERSATION,
    label: "By Conversation",
  },
];

export const PLATFORM_USAGE_PAGE_SIZE = 10;
export const TENANT_ID_PATH_PARAMETER = ":usageTenantId";
export const CONVERSATION_ID_PATH_PARAMETER = ":conversationId";

export const TENANTS_USAGE_TABLE_COLUMNS = "minmax(0, 1fr) 8rem 6rem 12rem";
export const USERS_USAGE_TABLE_COLUMNS =
  "minmax(0, 1fr) minmax(0, 1fr) 8rem 6rem";
export const CONVERSATIONS_USAGE_TABLE_COLUMNS =
  "minmax(0, 1.4fr) minmax(0, 1fr) minmax(0, 1fr) 8rem 6rem";

export const PLATFORM_USAGE_DESCRIPTION = "Platform AI usage.";
export const PLATFORM_AI_OFF_NOTICE =
  "Platform AI is off in this deployment. Usage recorded while it was on is still shown.";
export const PLATFORM_USAGE_EMPTY_HEADING =
  "No platform AI usage in this range";
export const PLATFORM_USAGE_EMPTY_DESCRIPTION =
  "When tenants without their own AI key use AI, the tokens the platform key pays for appear here, by day, tenant, user and conversation.";
