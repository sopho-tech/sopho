export const APP_URLS = {
  HOME: "/",
  SIGN_IN: "/signin",
  PROFILE: "/profile",
  CONNECTIONS: "/settings/connections",
  NEW_CONNECTION: "/settings/connection/new",
  AI_CONFIGURATIONS: "/settings/ai-configurations",
  NEW_CONVERSATION: "/conversational_analytics/",
} as const;

export const TENANT_HOME_URL_PATTERN = /\/t\/(?<tenantId>[0-9a-f-]{36})\/?$/;

export function tenantUrl(tenantId: string, path: string): string {
  return `/t/${tenantId}${path}`;
}

export function canvasNotebookUrl(tenantId: string, canvasId: string): string {
  return tenantUrl(tenantId, `/canvas/${canvasId}/notebook`);
}

export function canvasDashboardUrl(tenantId: string, canvasId: string): string {
  return tenantUrl(tenantId, `/canvas/${canvasId}/dashboard`);
}
