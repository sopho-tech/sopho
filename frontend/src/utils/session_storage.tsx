import { SessionStorageKey } from "src/constants/session_storage";

export function readSessionStorageValue(key: SessionStorageKey): string | null {
  return window.sessionStorage.getItem(key);
}

export function writeSessionStorageValue(
  key: SessionStorageKey,
  value: string,
): void {
  window.sessionStorage.setItem(key, value);
}

export function removeSessionStorageValue(key: SessionStorageKey): void {
  window.sessionStorage.removeItem(key);
}
