export const CONNECT_RESULT_PARAMETER = "connect_result";
export const CONNECTED_PROVIDER_PARAMETER = "provider";
export const CONNECTED_RESULT = "connected";
export const PASSWORD_CHANGED_MESSAGE =
  "Password changed. Your other sessions were signed out.";
export const CHANGE_PASSWORD_FORM_DEFAULT_VALUES = {
  current_password: "",
  new_password: "",
};
export const CONNECT_RESULT_MESSAGES: Record<string, string> = {
  email_mismatch:
    "That account's email doesn't match your Sopho email, so it wasn't connected.",
  identity_in_use:
    "That sign-in is already connected to another Sopho account.",
  already_connected: "That sign-in method is already connected.",
  email_not_verified:
    "The provider hasn't verified your email address, so it wasn't connected.",
  account_deactivated: "This account has been deactivated.",
  sso_expired: "That attempt expired or started in another browser. Try again.",
  sso_failed: "Connecting didn't complete. Try again.",
};
