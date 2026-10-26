import { EmailStatus } from "src/api/invitation/queries";
export const UNSTYLED_LIST_STYLE = { listStyle: "none", margin: 0, padding: 0 };

export const EMAIL_STATUS_LABELS: Record<EmailStatus, string> = {
  PENDING: "Email queued",
  SENDING: "Email sending",
  SENT: "Email sent",
  FAILED: "Email failed",
  CANCELLED: "Email cancelled",
};
