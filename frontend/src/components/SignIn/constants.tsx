export const SIGN_IN_ERROR_PARAMETER = "error";

export const SIGN_IN_FORM_DEFAULT_VALUES = { email: "", password: "" };

export const SIGN_IN_ERROR_MESSAGES: Record<string, string> = {
  sso_failed: "Single sign-on did not complete. Try again.",
  email_not_verified:
    "Your identity provider has not verified your email address, so it cannot be used to sign in.",
  sso_expired: "That sign-in expired or started in another browser. Try again.",
  password_account:
    "This email belongs to an account that signs in with a password. Sign in with your password, then connect single sign-on from your profile.",
  account_deactivated:
    "This account has been deactivated. Ask your Sopho administrator for help.",
};
