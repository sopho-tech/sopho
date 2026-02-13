export function getErrorSummary(formFieldMeta: any): string | undefined {
  const invalidCount = Object.values(formFieldMeta || {}).filter(
    (meta: any) => meta?.isValid === false
  ).length;

  if (invalidCount > 0) {
    return `${invalidCount} invalid field${invalidCount === 1 ? "" : "s"} need${invalidCount === 1 ? "s" : ""} to be fixed below`;
  }

  return undefined;
}
