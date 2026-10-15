import { afterEach, describe, expect, test, vi } from "vitest";
import {
  formatDate,
  formatRelativeTime,
  formatTimestamp,
} from "src/utils/timestamp_utils";

describe("timestamp formatting", () => {
  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllEnvs();
  });

  test("describes a timestamp relative to now", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-15T12:00:00Z"));

    expect(formatRelativeTime("2026-10-15T10:00:00Z")).toBe("2 hours ago");
  });

  test("formats dates and timestamps in the local time zone", () => {
    vi.stubEnv("TZ", "UTC");
    expect(formatDate("2026-10-15T22:30:00Z")).toBe("Oct 15, 2026");
    expect(formatTimestamp("2026-10-15T09:05:00Z")).toBe(
      "Oct 15, 2026 9:05 AM",
    );

    vi.stubEnv("TZ", "America/New_York");
    expect(formatDate("2026-10-16T02:30:00Z")).toBe("Oct 15, 2026");
    expect(formatTimestamp("2026-10-15T09:05:00Z")).toBe(
      "Oct 15, 2026 5:05 AM",
    );
  });

  test("returns null for a missing timestamp", () => {
    expect(formatRelativeTime(null)).toBeNull();
    expect(formatDate(null)).toBeNull();
    expect(formatTimestamp(null)).toBeNull();
  });
});
