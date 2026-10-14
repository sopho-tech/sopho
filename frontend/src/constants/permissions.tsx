export const PERMISSIONS = {
  CANVAS_CREATE: "canvas:create",
  CONVERSATION_CREATE: "conversation:create",
  CONNECTION_CREATE: "connection:create",
  AI_CONFIGURATION_WRITE: "ai_configuration:write",
  MEMBER_INVITE: "member:invite",
} as const;

export type Permission = (typeof PERMISSIONS)[keyof typeof PERMISSIONS];
