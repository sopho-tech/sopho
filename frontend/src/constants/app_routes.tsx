export const APP_ROUTES = {
  INDEX: "/",
  CANVAS: "/canvas/:id",
  CANVASES: "/canvas",
  SETTINGS: "/settings",
  SETTINGS_ROUTES: {
    CONNECTIONS: "/settings/connections",
    AI_CONFIGURATIONS: "/settings/ai-configurations",
  },
  CONNECTION_NEW: "/settings/connection/new",
  CONNECTION_EDIT: "/settings/connection/:id/edit",
  SIGN_IN: "/signin",
  PROFILE: "/profile",
  CONVERSATIONAL_ANALYTICS: "/conversational_analytics",
  CONVERSATIONAL_ANALYTICS_ROUTES: {
    INDEX: "/conversational_analytics/",
    CONVERSATION: "/conversational_analytics/conversation/:id",
  },
};
