const NUMBER_FORMAT = new Intl.NumberFormat("en-US");
const UTC_DAY_FORMAT = new Intl.DateTimeFormat("en-US", {
  month: "short",
  day: "numeric",
  year: "numeric",
  timeZone: "UTC",
});
const UTC_MOMENT_FORMAT = new Intl.DateTimeFormat("en-US", {
  month: "short",
  day: "numeric",
  hour: "2-digit",
  minute: "2-digit",
  hourCycle: "h23",
  timeZone: "UTC",
});

export function formatCount(count: number): string {
  return NUMBER_FORMAT.format(count);
}

export function formatUtcDay(day: string): string {
  return UTC_DAY_FORMAT.format(new Date(`${day}T00:00:00Z`));
}

export function formatUtcMoment(moment: string): string {
  return `${UTC_MOMENT_FORMAT.format(new Date(moment))} UTC`;
}
