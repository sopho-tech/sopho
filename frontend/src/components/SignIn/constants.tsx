export const SIGN_IN_ERROR_PARAMETER = "error";

export const SIGN_IN_ERROR_MESSAGES: Record<string, string> = {
  sso_failed: "Single sign-on did not complete. Try again.",
  email_not_verified:
    "Your identity provider has not verified your email address, so it cannot be used to sign in.",
  sso_expired: "That sign-in expired or started in another browser. Try again.",
  password_account:
    "This email belongs to an account that signs in with a password. Use your password to sign in.",
};
