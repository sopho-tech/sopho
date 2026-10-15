export const APP_URLS = {
  HOME: "/",
  SIGN_IN: "/signin",
  PROFILE: "/profile",
  CONNECTIONS: "/settings/connections",
  NEW_CONNECTION: "/settings/connection/new",
  AI_CONFIGURATIONS: "/settings/ai-configurations",
  NEW_CONVERSATION: "/conversational_analytics/",
} as const;

export function canvasNotebookUrl(canvasId: string): string {
  return `/canvas/${canvasId}/notebook`;
}

export function canvasDashboardUrl(canvasId: string): string {
  return `/canvas/${canvasId}/dashboard`;
}
