export const PLATFORM_PERMISSIONS = {
  TENANT_CREATE: "tenant:create",
  USER_MANAGE: "user:manage",
  TENANT_APPOINT_OWNER: "tenant:appoint_owner",
  EMAIL_MANAGE: "email:manage",
  TENANT_SUSPEND: "tenant:suspend",
  PLATFORM_AI_USAGE_READ: "platform_ai_usage:read",
} as const;

export type PlatformPermission =
  (typeof PLATFORM_PERMISSIONS)[keyof typeof PLATFORM_PERMISSIONS];
