import {
  CartesianGrid,
  Legend,
  Line,
  LineChart as RechartsLineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import commonStyles from "../ChartCommon.module.css";
import { getCSSVariable } from "src/utils/css_util";
import {
  CHART_MARGINS,
  createTooltipProps,
  createLegendProps,
  renderXAxisLabel,
  renderYAxisLabel,
} from "../ChartCommon";

export type LineChartType = "linear" | "monotone" | "step";

export type LineChartProps = {
  type?: LineChartType;
  xAxis: string;
  yAxis: string;
  data: Object[];
  xAxisTitle?: string;
  yAxisTitle?: string;
};

export function LineChart({
  type = "monotone",
  xAxis,
  yAxis,
  data,
  xAxisTitle,
  yAxisTitle,
}: LineChartProps) {
  return (
    <ResponsiveContainer
      width="100%"
      initialDimension={{ width: 1, height: 1 }}
      debounce={300}
      className={commonStyles.container}
    >
      <RechartsLineChart responsive data={data} margin={{ ...CHART_MARGINS }}>
        <CartesianGrid
          stroke={getCSSVariable("--color-grey-300")}
          strokeDasharray={"4 1 2"}
        />
        <Line
          type={type}
          dataKey={yAxis}
          stroke={getCSSVariable("--color-primary-500")}
          strokeWidth={1}
          name={yAxis}
          activeDot={{ r: 7 }}
        />
        <XAxis dataKey={xAxis} stroke={getCSSVariable("--color-grey-600")}>
          {renderXAxisLabel(xAxisTitle)}
        </XAxis>
        <YAxis stroke={getCSSVariable("--color-grey-600")}>
          {renderYAxisLabel(yAxisTitle)}
        </YAxis>
        <Legend {...createLegendProps("plainline")} />
        <Tooltip {...createTooltipProps({ strokeDasharray: "3 3" })} />
      </RechartsLineChart>
    </ResponsiveContainer>
  );
}
