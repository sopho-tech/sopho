import { useCallback, useMemo } from "react";
import { PieChart as RechartsPieChart, Pie, Label } from "recharts";
import {
  getPrimaryColorShades,
  ChartLegend,
  ChartTooltip,
  ChartContainer,
} from "../ChartCommon";

export type PieChartProps = {
  category: string;
  value: string;
  dimensions: string[];
  data: object[];
  sortOrder?: string;
};

function getDecimalPlaces(val: unknown): number {
  const str = String(val ?? "");
  const dot = str.indexOf(".");
  return dot === -1 ? 0 : str.length - dot - 1;
}

function calculateTotal(data: object[], valueKey: string): number {
  let maxDecimals = 0;
  const sum = data.reduce<number>((acc, item) => {
    const raw = (item as Record<string, unknown>)[valueKey];
    const num = typeof raw === "number" ? raw : parseFloat(String(raw ?? 0));
    if (Number.isFinite(num)) {
      maxDecimals = Math.max(maxDecimals, getDecimalPlaces(raw));
      return acc + num;
    }
    return acc;
  }, 0);
  const factor = 10 ** maxDecimals;
  return Math.round(sum * factor) / factor;
}

export const PieChart = ({ category, value, data }: PieChartProps) => {
  const colors = getPrimaryColorShades(data.length);

  const dataWithColors = data.map((item, index) => ({
    ...item,
    fill: colors[index],
  }));

  const totalCategoryValue = useMemo(
    () => calculateTotal(data, value),
    [data, value]
  );

  const renderLabelContent = useCallback(
    (props: { viewBox?: unknown }) => {
      const vb = (props.viewBox ?? {}) as
        | { cx?: number; cy?: number }
        | { x?: number; y?: number; width?: number; height?: number };
      const x =
        "cx" in vb && vb.cx != null
          ? vb.cx
          : "x" in vb && vb.width != null
            ? (vb.x ?? 0) + vb.width / 2
            : 0;
      const y =
        "cy" in vb && vb.cy != null
          ? vb.cy
          : "y" in vb && vb.height != null
            ? (vb.y ?? 0) + vb.height / 2
            : 0;
      return (
        <text
          x={x}
          y={y}
          textAnchor="middle"
          dominantBaseline="middle"
          fill="var(--color-grey-900)"
        >
          <tspan x={x} dy="-0.7em" fill="var(--color-grey-500)">
            Total
          </tspan>
          <tspan x={x} dy="1.5em">
            {totalCategoryValue}
          </tspan>
        </text>
      );
    },
    [totalCategoryValue]
  );

  return (
    <ChartContainer>
      <RechartsPieChart>
        <Pie
          data={dataWithColors}
          dataKey={value}
          cx="50%"
          cy="50%"
          innerRadius="60%"
          outerRadius="80%"
          nameKey={category}
          paddingAngle={1}
        >
          <Label position="center" fill="#666" content={renderLabelContent} />
        </Pie>
        <ChartLegend position="right" />
        <ChartTooltip />
      </RechartsPieChart>
    </ChartContainer>
  );
};
