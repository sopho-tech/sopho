import { EmailStatus } from "src/api/invitation/queries";
import { BadgeProps } from "src/components/design-system/Badge";

export const FIRST_PAGE = 1;
export const SEARCH_DEBOUNCE_MILLISECONDS = 300;

export const PLATFORM_ASIDE_STYLE = {
  backgroundColor: "var(--color-primary-100)",
};
export const PLATFORM_NAVIGATION_LIST_STYLE = {
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-2xs)",
  listStyle: "none",
  margin: 0,
  padding: 0,
} as const;
export const PLATFORM_NAVIGATION_ITEM_STYLE = {
  borderRadius: "var(--border-radius-lg)",
  transition: "var(--transition-bg-color)",
};
export const PLATFORM_ACTIVE_NAVIGATION_ITEM_STYLE = {
  ...PLATFORM_NAVIGATION_ITEM_STYLE,
  backgroundColor: "var(--color-grey-100)",
};
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
  FAILED: "yellow",
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
