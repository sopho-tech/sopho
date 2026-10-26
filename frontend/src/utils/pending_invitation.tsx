import { APP_ROUTES } from "src/constants/app_routes";
import {
  LINK_TOKEN_PARAMETER,
  PENDING_INVITATION_TOKEN_KEY,
} from "src/constants/session_storage";

export function linkTokenFromHash(hash: string): string | null {
  return new URLSearchParams(hash.replace(/^#/, "")).get(LINK_TOKEN_PARAMETER);
}

export function invitationPath(token: string): string {
  return `${APP_ROUTES.INVITATION}#${LINK_TOKEN_PARAMETER}=${token}`;
}

export function rememberPendingInvitation(token: string): void {
  window.sessionStorage.setItem(PENDING_INVITATION_TOKEN_KEY, token);
}

export function pendingInvitationToken(): string | null {
  return window.sessionStorage.getItem(PENDING_INVITATION_TOKEN_KEY);
}

export function forgetPendingInvitation(): void {
  window.sessionStorage.removeItem(PENDING_INVITATION_TOKEN_KEY);
}
