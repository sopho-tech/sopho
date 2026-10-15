import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";

dayjs.extend(relativeTime);

export function formatRelativeTime(timestamp: string | null): string | null {
  if (!timestamp) {
    return null;
  }
  return dayjs(timestamp).fromNow();
}

export function formatDate(timestamp: string | null): string | null {
  if (!timestamp) {
    return null;
  }
  return dayjs(timestamp).format("MMM D, YYYY");
}

export function formatTimestamp(timestamp: string | null): string | null {
  if (!timestamp) {
    return null;
  }
  return dayjs(timestamp).format("MMM D, YYYY h:mm A");
}
