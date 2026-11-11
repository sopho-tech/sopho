import { LINK_TOKEN_PARAMETER } from "src/constants/link_token";

export function linkTokenFromHash(hash: string): string | null {
  return new URLSearchParams(hash.replace(/^#/, "")).get(LINK_TOKEN_PARAMETER);
}

export function pathWithLinkToken(path: string, token: string): string {
  return `${path}#${LINK_TOKEN_PARAMETER}=${token}`;
}
