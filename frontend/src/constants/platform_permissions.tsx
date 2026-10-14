export const PLATFORM_PERMISSIONS = {
  TENANT_CREATE: "tenant:create",
} as const;

export type PlatformPermission =
  (typeof PLATFORM_PERMISSIONS)[keyof typeof PLATFORM_PERMISSIONS];
