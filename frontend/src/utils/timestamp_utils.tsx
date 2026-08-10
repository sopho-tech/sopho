import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import utc from "dayjs/plugin/utc";

dayjs.extend(utc);
dayjs.extend(relativeTime);

export function formatRelativeTime(timestamp: string | null): string | null {
  if (!timestamp) {
    return null;
  }
  return dayjs(timestamp).fromNow();
}

export function formatDate(
  timestamp: string | null,
  timezoneOffset: number = 330
): string | null {
  if (!timestamp) {
    return null;
  }
  return dayjs(timestamp).utcOffset(timezoneOffset).format("MMM D, YYYY");
}

export function formatTimestamp(
  timestamp: string | null,
  timezoneOffset: number = 330
): string | null {
  if (!timestamp) {
    return null;
  }
  return dayjs(timestamp)
    .utcOffset(timezoneOffset)
    .format("MMM D, YYYY h:mm A");
}
