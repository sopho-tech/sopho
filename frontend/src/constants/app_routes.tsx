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
    MEMBERS: "/settings/members",
  },
  CONNECTION_NEW: "/settings/connection/new",
  CONNECTION_EDIT: "/settings/connection/:id/edit",
  SIGN_IN: "/signin",
  PROFILE: "/profile",
  CONVERSATIONAL_ANALYTICS: "/conversational_analytics",
  CONVERSATIONAL_ANALYTICS_ROUTES: {
    INDEX: "/conversational_analytics/",
    CONVERSATION: "/conversational_analytics/conversation/:id",
    CONVERSATIONS: "/conversational_analytics/conversations",
  },
};
