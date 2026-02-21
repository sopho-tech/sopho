import { Flex, Text } from "src/components/design-system";
import styles from "src/components/Chart/ChartCommon.module.css";

export type MetricChartProps = {
  data: object[];
  precision?: number;
  suffix?: string;
  format?: "PERCENTAGE" | "CURRENCY" | "DEFAULT";
};

export const MetricChart = ({
  data,
  precision = 2,
  suffix,
  format = "DEFAULT",
}: MetricChartProps) => {
  if (data.length > 1) {
    throw new Error("too many rows");
  }
  if (Object.keys(data[0]).length > 1) {
    throw new Error("too many columns");
  }
  const raw = (Object.values(data[0])[0] as number).toFixed(precision);
  const base =
    format === "PERCENTAGE"
      ? `${raw}%`
      : format === "CURRENCY"
        ? `$${raw}`
        : raw;
  const formatted = suffix ? `${base} ${suffix}` : base;
  return (
    <Flex
      height="100%"
      width="100%"
      alignItems="center"
      justifyContent="center"
      className={styles.container}
    >
      <Text fontSize="3xl">{formatted}</Text>
    </Flex>
  );
};
