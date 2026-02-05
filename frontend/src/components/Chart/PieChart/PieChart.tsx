import {
  ResponsiveContainer,
  PieChart as RechartsPieChart,
  Pie,
  Legend,
  Tooltip,
} from "recharts";
import commonStyles from "../ChartCommon.module.css";
import {
  getPrimaryColorShades,
  createTooltipProps,
  createLegendProps,
} from "../ChartCommon";

export type PieChartProps = {
  category: string;
  value: string;
  dimensions: string[];
  data: Object[];
  sortOrder?: string;
};

export const PieChart = ({ category, value, data }: PieChartProps) => {
  const colors = getPrimaryColorShades(data.length);

  const dataWithColors = data.map((item, index) => ({
    ...item,
    fill: colors[index],
  }));

  return (
    <ResponsiveContainer
      width="100%"
      initialDimension={{ width: 1, height: 1 }}
      debounce={300}
      className={commonStyles.container}
    >
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
        />
        <Legend {...createLegendProps("rect", { paddingTop: "1rem" })} />
        <Tooltip {...createTooltipProps({ fill: "rgba(0, 0, 0, 0.05)" })} />
      </RechartsPieChart>
    </ResponsiveContainer>
  );
};
