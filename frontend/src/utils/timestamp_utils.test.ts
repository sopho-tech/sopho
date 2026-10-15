import { afterEach, describe, expect, test, vi } from "vitest";
import {
  formatDate,
  formatRelativeTime,
  formatTimestamp,
} from "src/utils/timestamp_utils";

describe("timestamp formatting", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  test("describes a timestamp relative to now", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-15T12:00:00Z"));

    expect(formatRelativeTime("2026-10-15T10:00:00Z")).toBe("2 hours ago");
  });

  test("formats dates and timestamps in the requested offset", () => {
    expect(formatDate("2026-10-15T22:30:00Z", 0)).toBe("Oct 15, 2026");
    expect(formatDate("2026-10-15T22:30:00Z", 330)).toBe("Oct 16, 2026");
    expect(formatTimestamp("2026-10-15T09:05:00Z", 0)).toBe(
      "Oct 15, 2026 9:05 AM",
    );
  });

  test("returns null for a missing timestamp", () => {
    expect(formatRelativeTime(null)).toBeNull();
    expect(formatDate(null)).toBeNull();
    expect(formatTimestamp(null)).toBeNull();
  });
});
