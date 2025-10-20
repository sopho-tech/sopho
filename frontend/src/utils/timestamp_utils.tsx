import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";

dayjs.extend(utc);

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
