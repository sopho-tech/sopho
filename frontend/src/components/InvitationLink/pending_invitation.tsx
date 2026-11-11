import { APP_ROUTES } from "src/constants/app_routes";
import { SESSION_STORAGE_KEYS } from "src/constants/session_storage";
import { pathWithLinkToken } from "src/utils/link_token";
import {
  readSessionStorageValue,
  removeSessionStorageValue,
  writeSessionStorageValue,
} from "src/utils/session_storage";

export function invitationPath(token: string): string {
  return pathWithLinkToken(APP_ROUTES.INVITATION, token);
}

export function rememberPendingInvitation(token: string): void {
  writeSessionStorageValue(
    SESSION_STORAGE_KEYS.PENDING_INVITATION_TOKEN,
    token,
  );
}

export function pendingInvitationToken(): string | null {
  return readSessionStorageValue(SESSION_STORAGE_KEYS.PENDING_INVITATION_TOKEN);
}

export function forgetPendingInvitation(): void {
  removeSessionStorageValue(SESSION_STORAGE_KEYS.PENDING_INVITATION_TOKEN);
}
