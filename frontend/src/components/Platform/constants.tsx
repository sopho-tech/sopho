import { EmailStatus } from "src/api/invitation/queries";
import { TenantStatus } from "src/api/platform/queries";
import { BadgeProps } from "src/components/design-system/Badge";

export const FIRST_PAGE = 1;
export const SEARCH_DEBOUNCE_MILLISECONDS = 300;

export const SEARCH_INPUT_STYLE = { maxWidth: "24rem" };

export const EMAIL_STATUS_NAMES: Record<EmailStatus, string> = {
  PENDING: "Queued",
  SENDING: "Sending",
  SENT: "Sent",
  FAILED: "Failed",
  CANCELLED: "Cancelled",
};

export const EMAIL_STATUS_BADGE_VARIANTS: Record<
  EmailStatus,
  NonNullable<BadgeProps["variant"]>
> = {
  PENDING: "blue",
  SENDING: "blue",
  SENT: "green",
  FAILED: "red",
  CANCELLED: "subtle",
};

export type EmailStatusFilter = {
  value: string;
  label: string;
  status: EmailStatus | null;
};

export const ALL_EMAIL_STATUSES_FILTER: EmailStatusFilter = {
  value: "ALL",
  label: "All statuses",
  status: null,
};

export const EMAIL_STATUS_FILTERS: EmailStatusFilter[] = [
  ALL_EMAIL_STATUSES_FILTER,
  { value: "FAILED", label: "Failed", status: "FAILED" },
  { value: "PENDING", label: "Queued", status: "PENDING" },
  { value: "SENDING", label: "Sending", status: "SENDING" },
  { value: "SENT", label: "Sent", status: "SENT" },
  { value: "CANCELLED", label: "Cancelled", status: "CANCELLED" },
];

export const USER_ACTIONS_LABEL = "User actions";

export const USERS_TABLE_COLUMNS =
  "minmax(0, 1.4fr) minmax(0, 0.8fr) minmax(0, 1.2fr) 2.5rem";
export const TENANTS_TABLE_COLUMNS = "minmax(0, 1fr) 8rem 10rem 10rem 7rem";
export const SUSPEND_TENANT_LABEL = "Suspend";
export const RESTORE_TENANT_LABEL = "Restore";
export const SUSPEND_TENANT_DESCRIPTION =
  "Nobody can use this tenant until it is restored. Its data, members and invitations stay as they are.";
export const RESTORE_TENANT_DESCRIPTION =
  "Members can use this tenant again, with everything as it was.";
export const TENANT_STATUS_NAMES: Record<TenantStatus, string> = {
  ACTIVE: "Active",
  INACTIVE: "Inactive",
  SUSPENDED: "Suspended",
};
export const TENANT_STATUS_BADGE_VARIANTS: Record<
  TenantStatus,
  NonNullable<BadgeProps["variant"]>
> = {
  ACTIVE: "green",
  INACTIVE: "subtle",
  SUSPENDED: "yellow",
};
export const EMAILS_TABLE_COLUMNS =
  "minmax(0, 1.2fr) minmax(0, 1.4fr) 9rem 6rem 5rem 10rem 2.5rem";
export const EMAIL_ACTIONS_LABEL = "Email actions";

export const VISIBLE_TENANT_COUNT = 2;
export const TENANT_ROLE_GRID_STYLE = {
  display: "grid",
  gridTemplateColumns: "minmax(0, max-content) max-content",
  alignItems: "center",
  columnGap: "var(--space-sm)",
  rowGap: "var(--space-2xs)",
};

export const HTML_EMAIL_FORMAT = "HTML";
export const TEXT_EMAIL_FORMAT = "TEXT";
export const EMAIL_FORMATS = [
  { value: HTML_EMAIL_FORMAT, label: "HTML" },
  { value: TEXT_EMAIL_FORMAT, label: "Plain text" },
];

export const EMAIL_TEMPLATE_LIST_STYLE = { width: "16rem", flexShrink: 0 };
export const EMAIL_TEMPLATE_PREVIEW_STYLE = { flex: 1, minWidth: 0 };
