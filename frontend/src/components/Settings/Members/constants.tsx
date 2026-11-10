import { EmailStatus } from "src/api/invitation/queries";

export const EMAIL_STATUS_LABELS: Record<EmailStatus, string> = {
  PENDING: "Queued",
  SENDING: "Sending",
  SENT: "Sent",
  FAILED: "Failed",
  CANCELLED: "Cancelled",
};

export const EMAIL_STATUS_BADGE_VARIANTS: Record<
  EmailStatus,
  "subtle" | "green" | "red"
> = {
  PENDING: "subtle",
  SENDING: "subtle",
  SENT: "green",
  FAILED: "red",
  CANCELLED: "subtle",
};

export const MEMBERS_TABLE_COLUMNS = "minmax(0, 1fr) 10rem 2.5rem";
export const INVITATIONS_TABLE_COLUMNS =
  "minmax(0, 1.5fr) 6rem minmax(0, 1fr) 6rem 9rem 2.5rem";
export const MEMBER_ACTIONS_LABEL = "Member actions";
export const INVITATION_ACTIONS_LABEL = "Invitation actions";

export const MEMBERS_TAB = "members";
export const INVITATIONS_TAB = "invitations";
