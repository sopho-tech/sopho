export const SEARCH_PARAMS = {
  CELL: "cell",
};

export const TENANT_ROUTE_PARAMETER = "tenantId";

export const APP_ROUTES = {
  INDEX: "/",
  ONBOARDING: "/onboarding",
  TENANT: "/t/:tenantId",
  CANVAS: "/canvas/:id",
  CANVASES: "/canvas",
  CANVAS_ROUTES: {
    NOTEBOOK: "notebook",
    DASHBOARD: "dashboard",
  },
  SETTINGS: "/settings",
  SETTINGS_ROUTES: {
    CONNECTIONS: "/settings/connections",
    AI_CONFIGURATIONS: "/settings/ai-configurations",
    AI_USAGE: "/settings/ai-usage",
    AI_USAGE_SUMMARY: "/settings/ai-usage/summary",
    AI_USAGE_BY_USER: "/settings/ai-usage/by-user",
    AI_USAGE_BY_WORK: "/settings/ai-usage/by-work",
    AI_USAGE_BY_CONVERSATION: "/settings/ai-usage/by-conversation",
    AI_USAGE_CONVERSATION: "/settings/ai-usage/by-conversation/:conversationId",
    MEMBERS: "/settings/members",
    INVITATIONS: "/settings/invitations",
  },
  PLATFORM: "/platform",
  PLATFORM_ROUTES: {
    USERS: "/platform/users",
    TENANTS: "/platform/tenants",
    AI_USAGE: "/platform/ai-usage",
    AI_USAGE_SUMMARY: "/platform/ai-usage/summary",
    AI_USAGE_BY_TENANT: "/platform/ai-usage/by-tenant",
    AI_USAGE_BY_USER: "/platform/ai-usage/by-user",
    AI_USAGE_BY_WORK: "/platform/ai-usage/by-work",
    AI_USAGE_BY_CONVERSATION: "/platform/ai-usage/by-conversation",
    AI_USAGE_CONVERSATION:
      "/platform/ai-usage/by-conversation/:usageTenantId/:conversationId",
    EMAILS: "/platform/emails",
    SENT_EMAILS: "/platform/emails/sent",
    EMAIL_TEMPLATES: "/platform/emails/templates",
  },
  CONNECTION_NEW: "/settings/connection/new",
  CONNECTION_EDIT: "/settings/connection/:id/edit",
  SIGN_IN: "/signin",
  INVITATION: "/invitation",
  FORGOT_PASSWORD: "/forgot-password",
  RESET_PASSWORD: "/reset-password",
  PROFILE: "/profile",
  CONVERSATIONAL_ANALYTICS: "/conversational_analytics",
  CONVERSATIONAL_ANALYTICS_ROUTES: {
    INDEX: "/conversational_analytics/",
    CONVERSATION: "/conversational_analytics/conversation/:id",
    CONVERSATIONS: "/conversational_analytics/conversations",
  },
};
