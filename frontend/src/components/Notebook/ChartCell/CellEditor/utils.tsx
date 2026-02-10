import { FormFieldType, FormField } from "src/components/design-system/Form";
import { ChartType } from "src/components/Chart";
import { Icon, Text, Flex } from "src/components/design-system";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import type {
  BarChartContent,
  LineChartContent,
  PieChartContent,
  ChartContent,
} from "../../Cell/dto";

type SelectOption = {
  value: string;
  label: string | React.ReactNode;
  textValue?: string;
};

type FormOptionsContext = {
  cellOptions: SelectOption[];
  chartOptions: SelectOption[];
  xAxisColumnOptions: SelectOption[];
  yAxisColumnOptions: SelectOption[];
  yAxisAggregateFunctionsOptions: SelectOption[];
};

const InfoTooltip = ({ message }: { message: string }) => (
  <div className={CellEditorStyle.infoTooltipMessage}>{message}</div>
);

const createSourceCellField = (
  cellOptions: SelectOption[],
  selectedValue?: string
) => ({
  key: "cell_id",
  name: "Source Cell",
  required: true,
  errorMessage: "Please select the source cell",
  type: FormFieldType.SELECT,
  options: cellOptions,
  selectedValue,
  infoIconToolTipMessage: (
    <InfoTooltip message="Select the source SQL Cell whose query you want to visualize" />
  ),
});

const createChartTypeField = (
  chartOptions: SelectOption[],
  selectedValue?: string
) => ({
  key: "chart_type",
  name: "Chart Type",
  required: true,
  errorMessage: "Please select the type of chart",
  type: FormFieldType.SELECT,
  options: chartOptions,
  selectedValue,
  infoIconToolTipMessage: (
    <InfoTooltip message="Select the type of chart you want to visualize data as" />
  ),
});

const SORT_ORDER_OPTIONS = [
  { value: "NONE", label: "None" },
  { value: "ASC", label: "Ascending" },
  { value: "DESC", label: "Descending" },
];

const ORIENTATION_OPTIONS = [
  { value: "VERTICAL", label: "Vertical" },
  { value: "HORIZONTAL", label: "Horizontal" },
];

const VISIBILITY_OPTIONS = [
  { value: "SHOW", label: "Show" },
  { value: "HIDE", label: "Hide" },
];

export function getBarChartFormFields(
  chartContent: ChartContent | null,
  options: FormOptionsContext
): FormField[] {
  const barChartContent = chartContent as BarChartContent;
  const {
    cellOptions,
    chartOptions,
    xAxisColumnOptions,
    yAxisColumnOptions,
    yAxisAggregateFunctionsOptions,
  } = options;

  return [
    createSourceCellField(cellOptions, chartContent?.cell_id),
    createChartTypeField(chartOptions, chartContent?.chart_type),
    {
      key: "x_axis_settings",
      name: (
        <Flex direction="row" gap="2xs" justifyContent="center">
          <Icon type="swap_horiz" color="default" />
          <Text fontSize="sm">X-axis</Text>
        </Flex>
      ),
      required: false,
      errorMessage: "",
      type: FormFieldType.COLLAPSIBLE,
      collapsibleConfig: {
        fields: [
          {
            key: "x_axis",
            name: "Column",
            required: true,
            errorMessage: "Please select the column for x-axis",
            type: FormFieldType.SELECT,
            options: xAxisColumnOptions,
            selectedValue: barChartContent?.x_axis,
            infoIconToolTipMessage: (
              <InfoTooltip message="Select the column from the SQL query's output to be x-axis" />
            ),
          },
          {
            key: "x_axis_title",
            name: "Title",
            required: false,
            errorMessage: "",
            type: FormFieldType.INPUT,
            defaultValue: barChartContent?.x_axis_title || "",
            placeholder: "Enter x-axis title",
            infoIconToolTipMessage: (
              <InfoTooltip message="Optional title to display for the x-axis" />
            ),
          },
        ],
      },
    },
    {
      key: "y_axis_settings",
      name: (
        <Flex direction="row" gap="2xs" justifyContent="center">
          <Icon type="swap_vert" color="default" />
          <Text fontSize="sm">Y-axis</Text>
        </Flex>
      ),
      required: false,
      errorMessage: "",
      type: FormFieldType.COLLAPSIBLE,
      collapsibleConfig: {
        fields: [
          {
            key: "y_axis",
            name: "Column",
            required: true,
            errorMessage: "Please select the column for y-axis",
            type: FormFieldType.SELECT,
            options: yAxisColumnOptions,
            selectedValue: barChartContent?.y_axis,
            infoIconToolTipMessage: (
              <InfoTooltip message="Select the column from the SQL query's output for y-axis" />
            ),
          },
          {
            key: "y_axis_aggregate_function",
            name: "Aggregate by",
            required: true,
            errorMessage:
              "Please select the aggregate function for y-axis column",
            type: FormFieldType.SELECT,
            options: yAxisAggregateFunctionsOptions,
            selectedValue: barChartContent?.y_axis_aggregate_function,
            infoIconToolTipMessage: (
              <InfoTooltip message="Select the function to aggregate the y-axis column" />
            ),
          },
          {
            key: "y_axis_sort_order",
            name: "Sort Order",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: SORT_ORDER_OPTIONS,
            selectedValue: barChartContent?.y_axis_sort_order || "NONE",
            infoIconToolTipMessage: (
              <InfoTooltip message="Specify how the chart should be sorted according to y-axis" />
            ),
          },
          {
            key: "y_axis_title",
            name: "Title",
            required: false,
            errorMessage: "",
            type: FormFieldType.INPUT,
            defaultValue: barChartContent?.y_axis_title || "",
            placeholder: "Enter y-axis title",
            infoIconToolTipMessage: (
              <InfoTooltip message="Optional title to display for the y-axis" />
            ),
          },
        ],
      },
    },
    {
      key: "display_settings",
      name: (
        <Flex direction="row" gap="xs" alignItems="center">
          <Icon type="settings" color="default" />
          <Text fontSize="sm">Display</Text>
        </Flex>
      ),
      required: false,
      errorMessage: "",
      type: FormFieldType.COLLAPSIBLE,
      collapsibleConfig: {
        fields: [
          {
            key: "orientation",
            name: "Orientation",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: ORIENTATION_OPTIONS,
            selectedValue: barChartContent?.orientation || "VERTICAL",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose how the chart should be layed out" />
            ),
          },
          {
            key: "x_axis_tick_show",
            name: "X-Axis Tick",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: VISIBILITY_OPTIONS,
            selectedValue: barChartContent?.x_axis_tick_show ?? "SHOW",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose whether to show the ticks on x-axis" />
            ),
          },
          {
            key: "y_axis_tick_show",
            name: "Y-Axis Tick",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: VISIBILITY_OPTIONS,
            selectedValue: barChartContent?.y_axis_tick_show ?? "SHOW",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose whether to show the ticks on y-axis" />
            ),
          },
          {
            key: "axis_minor_tick_show",
            name: "Minor Tick",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: VISIBILITY_OPTIONS,
            selectedValue: barChartContent?.axis_minor_tick_show || "SHOW",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose whether to show the minor ticks in x-axis" />
            ),
          },
        ],
      },
    },
  ];
}

export function getLineChartFormFields(
  chartContent: ChartContent | null,
  options: FormOptionsContext
): FormField[] {
  const lineChartContent = chartContent as LineChartContent;
  const {
    cellOptions,
    chartOptions,
    xAxisColumnOptions,
    yAxisColumnOptions,
    yAxisAggregateFunctionsOptions,
  } = options;

  return [
    createSourceCellField(cellOptions, chartContent?.cell_id),
    createChartTypeField(chartOptions, chartContent?.chart_type),
    {
      key: "x_axis_settings",
      name: (
        <Flex direction="row" gap="2xs" justifyContent="center">
          <Icon type="swap_horiz" color="default" />
          <Text fontSize="sm">X-axis</Text>
        </Flex>
      ),
      required: false,
      errorMessage: "",
      type: FormFieldType.COLLAPSIBLE,
      collapsibleConfig: {
        fields: [
          {
            key: "x_axis",
            name: "Column",
            required: true,
            errorMessage: "Please select the column for x-axis",
            type: FormFieldType.SELECT,
            options: xAxisColumnOptions,
            selectedValue: lineChartContent?.x_axis,
            infoIconToolTipMessage: (
              <InfoTooltip message="Select the column from the SQL query's output to be x-axis" />
            ),
          },
          {
            key: "x_axis_title",
            name: "Title",
            required: false,
            errorMessage: "",
            type: FormFieldType.INPUT,
            defaultValue: lineChartContent?.x_axis_title || "",
            placeholder: "Enter x-axis title",
            infoIconToolTipMessage: (
              <InfoTooltip message="Optional title to display for the x-axis" />
            ),
          },
        ],
      },
    },
    {
      key: "y_axis_settings",
      name: (
        <Flex direction="row" gap="2xs" justifyContent="center">
          <Icon type="swap_vert" color="default" />
          <Text fontSize="sm">Y-axis</Text>
        </Flex>
      ),
      required: false,
      errorMessage: "",
      type: FormFieldType.COLLAPSIBLE,
      collapsibleConfig: {
        fields: [
          {
            key: "y_axis",
            name: "Column",
            required: true,
            errorMessage: "Please select the column for y-axis",
            type: FormFieldType.SELECT,
            options: yAxisColumnOptions,
            selectedValue: lineChartContent?.y_axis,
            infoIconToolTipMessage: (
              <InfoTooltip message="Select the column from the SQL query's output for y-axis" />
            ),
          },
          {
            key: "y_axis_aggregate_function",
            name: "Aggregate by",
            required: true,
            errorMessage:
              "Please select the aggregate function for y-axis column",
            type: FormFieldType.SELECT,
            options: yAxisAggregateFunctionsOptions,
            selectedValue: lineChartContent?.y_axis_aggregate_function,
            infoIconToolTipMessage: (
              <InfoTooltip message="Select the function to aggregate the y-axis column" />
            ),
          },
          {
            key: "y_axis_sort_order",
            name: "Sort Order",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: SORT_ORDER_OPTIONS,
            selectedValue: lineChartContent?.y_axis_sort_order || "NONE",
            infoIconToolTipMessage: (
              <InfoTooltip message="Specify how the chart should be sorted according to y-axis" />
            ),
          },
          {
            key: "y_axis_title",
            name: "Title",
            required: false,
            errorMessage: "",
            type: FormFieldType.INPUT,
            defaultValue: lineChartContent?.y_axis_title || "",
            placeholder: "Enter y-axis title",
            infoIconToolTipMessage: (
              <InfoTooltip message="Optional title to display for the y-axis" />
            ),
          },
        ],
      },
    },
    {
      key: "display_settings",
      name: (
        <Flex direction="row" gap="xs" alignItems="center">
          <Icon type="settings" color="default" />
          <Text fontSize="sm">Display</Text>
        </Flex>
      ),
      required: false,
      errorMessage: "",
      type: FormFieldType.COLLAPSIBLE,
      collapsibleConfig: {
        fields: [
          {
            key: "orientation",
            name: "Orientation",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: ORIENTATION_OPTIONS,
            selectedValue: lineChartContent?.orientation || "VERTICAL",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose how the chart should be layed out" />
            ),
          },
          {
            key: "x_axis_tick_show",
            name: "X-Axis Tick",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: VISIBILITY_OPTIONS,
            selectedValue: lineChartContent?.x_axis_tick_show ?? "SHOW",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose whether to show the ticks on x-axis" />
            ),
          },
          {
            key: "y_axis_tick_show",
            name: "Y-Axis Tick",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: VISIBILITY_OPTIONS,
            selectedValue: lineChartContent?.y_axis_tick_show ?? "SHOW",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose whether to show the ticks on y-axis" />
            ),
          },
          {
            key: "axis_minor_tick_show",
            name: "Minor Tick",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: VISIBILITY_OPTIONS,
            selectedValue: lineChartContent?.axis_minor_tick_show || "SHOW",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose whether to show the minor ticks in x-axis" />
            ),
          },
          {
            key: "show_dots",
            name: "Show Dots",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: VISIBILITY_OPTIONS,
            selectedValue: lineChartContent?.show_dots ?? "SHOW",
            infoIconToolTipMessage: (
              <InfoTooltip message="Choose whether to show dots on the line" />
            ),
          },
        ],
      },
    },
  ];
}

export function getPieChartFormFields(
  chartContent: ChartContent | null,
  options: FormOptionsContext
): FormField[] {
  const pieChartContent = chartContent as PieChartContent;
  const {
    cellOptions,
    chartOptions,
    xAxisColumnOptions,
    yAxisColumnOptions,
    yAxisAggregateFunctionsOptions,
  } = options;

  return [
    createSourceCellField(cellOptions, chartContent?.cell_id),
    createChartTypeField(chartOptions, chartContent?.chart_type),
    {
      key: "category",
      name: "Category Name",
      required: true,
      errorMessage: "Please select the category column",
      type: FormFieldType.SELECT,
      options: yAxisColumnOptions,
      selectedValue: pieChartContent?.category,
      infoIconToolTipMessage: (
        <InfoTooltip message="Select the column to use as category labels" />
      ),
    },
    {
      key: "value",
      name: "Value Name",
      required: true,
      errorMessage: "Please select the value column",
      type: FormFieldType.SELECT,
      options: xAxisColumnOptions,
      selectedValue: pieChartContent?.value,
      infoIconToolTipMessage: (
        <InfoTooltip message="Select the column to use as values" />
      ),
    },
    {
      key: "aggregate_function",
      name: "Aggregate by",
      required: true,
      errorMessage: "Please select the aggregate function for y-axis column",
      type: FormFieldType.SELECT,
      options: yAxisAggregateFunctionsOptions,
      selectedValue: pieChartContent?.aggregate_function,
      infoIconToolTipMessage: (
        <InfoTooltip message="Select the function to aggregate the values" />
      ),
    },
  ];
}

export function getDefaultFormFields(
  chartContent: ChartContent | null,
  options: FormOptionsContext
): FormField[] {
  const { cellOptions, chartOptions } = options;

  return [
    createSourceCellField(cellOptions, chartContent?.cell_id),
    createChartTypeField(chartOptions, chartContent?.chart_type),
  ];
}

export function getFormFieldsByChartType(
  chartType: ChartType | null,
  chartContent: ChartContent | null,
  options: FormOptionsContext
) {
  switch (chartType) {
    case ChartType.BAR:
      return getBarChartFormFields(chartContent, options);
    case ChartType.LINE:
      return getLineChartFormFields(chartContent, options);
    case ChartType.PIE:
      return getPieChartFormFields(chartContent, options);
    default:
      return getDefaultFormFields(chartContent, options);
  }
}

export function extractBarChartFormData(formData: FormData): BarChartContent {
  return {
    x_axis: (formData.get("x_axis") as string) || "",
    y_axis: (formData.get("y_axis") as string) || "",
    chart_type: (formData.get("chart_type") as string) || undefined,
    cell_id: (formData.get("cell_id") as string) || undefined,
    orientation: (formData.get("orientation") as string) || undefined,
    y_axis_aggregate_function:
      (formData.get("y_axis_aggregate_function") as string) || undefined,
    y_axis_sort_order:
      (formData.get("y_axis_sort_order") as string) || undefined,
    x_axis_tick_show: (formData.get("x_axis_tick_show") as string) || undefined,
    y_axis_tick_show: (formData.get("y_axis_tick_show") as string) || undefined,
    axis_minor_tick_show:
      (formData.get("axis_minor_tick_show") as string) || undefined,
    x_axis_title: (formData.get("x_axis_title") as string) || undefined,
    y_axis_title: (formData.get("y_axis_title") as string) || undefined,
  };
}

export function extractLineChartFormData(formData: FormData): LineChartContent {
  return {
    x_axis: (formData.get("x_axis") as string) || "",
    y_axis: (formData.get("y_axis") as string) || "",
    chart_type: (formData.get("chart_type") as string) || undefined,
    cell_id: (formData.get("cell_id") as string) || undefined,
    orientation: (formData.get("orientation") as string) || undefined,
    y_axis_aggregate_function:
      (formData.get("y_axis_aggregate_function") as string) || undefined,
    y_axis_sort_order:
      (formData.get("y_axis_sort_order") as string) || undefined,
    x_axis_tick_show: (formData.get("x_axis_tick_show") as string) || undefined,
    y_axis_tick_show: (formData.get("y_axis_tick_show") as string) || undefined,
    axis_minor_tick_show:
      (formData.get("axis_minor_tick_show") as string) || undefined,
    show_dots: (formData.get("show_dots") as string) || "SHOW",
    x_axis_title: (formData.get("x_axis_title") as string)?.trim() || undefined,
    y_axis_title: (formData.get("y_axis_title") as string)?.trim() || undefined,
  };
}

export function extractPieChartFormData(formData: FormData): PieChartContent {
  return {
    chart_type: formData.get("chart_type") as string,
    cell_id: formData.get("cell_id") as string,
    category: formData.get("category") as string,
    value: formData.get("value") as string,
    aggregate_function: formData.get("aggregate_function") as string,
  };
}

export function extractChartFormData(
  chartType: ChartType | null,
  formData: FormData
): BarChartContent | LineChartContent | PieChartContent {
  switch (chartType) {
    case ChartType.BAR:
      return extractBarChartFormData(formData);
    case ChartType.LINE:
      return extractLineChartFormData(formData);
    case ChartType.PIE:
      return extractPieChartFormData(formData);
    default:
      throw Error("Chart type is not supported");
  }
}
