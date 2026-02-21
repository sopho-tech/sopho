export function validateMetricChartData(data: object[]): string | null {
  if (data.length > 1)
    return "Metric charts display a single value. Use a query that returns exactly one row.";
  if (data.length === 0 || !data[0])
    return "Query returned no data. Ensure your SQL returns at least one row.";
  if (Object.keys(data[0]).length > 1)
    return "Metric charts display a single value. Use a query that returns exactly one column.";
  return null;
}
