export const SESSION_STORAGE_KEYS = {
  PENDING_INVITATION_TOKEN: "pending_invitation_token",
} as const;

export type SessionStorageKey =
  (typeof SESSION_STORAGE_KEYS)[keyof typeof SESSION_STORAGE_KEYS];
