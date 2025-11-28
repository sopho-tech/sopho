import { FormField, FormFieldType } from "src/components/design-system/Form";
import { useUpdateCell, useCell } from "src/api/cell/queries";
import {
  CellDto,
  getChartContent,
  serializeChartContent,
  useHandleExecuteCell,
} from "src/components/Notebooks/Notebook/Cell";
import CellEditorStyle from "src/components/Notebooks/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import { Button } from "src/components/design-system/Button";
import {
  useSourceCellExecution,
  useFormOptions,
} from "src/components/Notebooks/Notebook/ChartCell/CellEditor/hooks";
import { Form, Icon, Text, Flex } from "src/components/design-system";

export function CellEditor({ cellId }: { cellId: string }) {
  const cellQuery = useCell(cellId);
  const updateCellMutation = useUpdateCell();
  const chartContent = cellQuery.data ? getChartContent(cellQuery.data) : null;
  const { setSourceCellId, sourceCellOutput } = useSourceCellExecution(
    cellId,
    chartContent
  );
  const handleExecuteCell = useHandleExecuteCell();
  const {
    cellOptions,
    chartOptions,
    xAxisColumnOptions,
    yAxisColumnOptions,
    yAxisAggregateFunctionsOptions,
  } = useFormOptions(sourceCellOutput);

  function handleSubmit(formData: FormData) {
    if (!cellQuery.data) throw Error("Cell query data is empty");
    const chartContent = {
      x_axis: (formData.get("x_axis") as string) || "",
      y_axis: (formData.get("y_axis") as string) || "",
      chart_type: (formData.get("chart_type") as string) || undefined,
      cell_id: (formData.get("cell_id") as string) || undefined,
      orientation: (formData.get("orientation") as string) || undefined,
      y_axis_aggregate_function:
        (formData.get("y_axis_aggregate_function") as string) || undefined,
      y_axis_sort_order:
        (formData.get("y_axis_sort_order") as string) || undefined,
      axis_tick_show: (formData.get("axis_tick_show") as string) || undefined,
      axis_minor_tick_show:
        (formData.get("axis_minor_tick_show") as string) || undefined,
    };
    const cellDto: CellDto = {
      ...cellQuery.data,
      content: serializeChartContent(chartContent),
    };
    updateCellMutation.mutate({ cellId, payload: cellDto });
  }

  function handleCancel() {}

  function handleChange(_: FormData, fieldName: string, value: string) {
    if (fieldName === "cell_id") {
      setSourceCellId(value);
    }
  }

  const fields: FormField[] = [
    {
      key: "cell_id",
      name: "Source Cell",
      required: true,
      errorMessage: "Please select the source cell",
      type: FormFieldType.SELECT,
      options: cellOptions,
      selectedValue: chartContent?.cell_id,
    },
    {
      key: "chart_type",
      name: "Chart Type",
      required: true,
      errorMessage: "Please select the type of chart",
      type: FormFieldType.SELECT,
      options: chartOptions,
      selectedValue: chartContent?.chart_type,
    },
    {
      key: "x_axis_settings",
      name: (
        <Flex direction="row" gap="2xs" justifyContent="center">
          <Icon type="swap_horiz" color="default" />
          <Text>X-axis</Text>
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
            selectedValue: chartContent?.x_axis,
          },
        ],
      },
    },
    {
      key: "y_axis_settings",
      name: (
        <Flex direction="row" gap="2xs" justifyContent="center">
          <Icon type="swap_vert" color="default" />
          <Text>Y-axis</Text>
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
            selectedValue: chartContent?.y_axis,
          },
          {
            key: "y_axis_aggregate_function",
            name: "Aggregate by",
            required: true,
            errorMessage: "Please select the column for y-axis",
            type: FormFieldType.SELECT,
            options: yAxisAggregateFunctionsOptions,
            selectedValue: chartContent?.y_axis_aggregate_function,
          },
          {
            key: "y_axis_sort_order",
            name: "Sort Order",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: [
              { value: "NONE", label: "None" },
              { value: "ASC", label: "Ascending" },
              { value: "DESC", label: "Descending" },
            ],
            selectedValue: chartContent?.y_axis_sort_order || "NONE",
          },
        ],
      },
    },
    {
      key: "display_settings",
      name: (
        <Flex direction="row" gap="xs" alignItems="center">
          <Icon type="settings" color="default" />
          <Text>Display</Text>
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
            options: [
              { value: "VERTICAL", label: "Vertical" },
              { value: "HORIZONTAL", label: "Horizontal" },
            ],
            selectedValue: chartContent?.orientation || "VERTICAL",
          },
          {
            key: "axis_tick_show",
            name: "Axis Tick",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: [
              { value: "SHOW", label: "Show" },
              { value: "HIDE", label: "Hide" },
            ],
            selectedValue: chartContent?.axis_tick_show || "SHOW",
          },
          {
            key: "axis_minor_tick_show",
            name: "Minor Tick",
            required: false,
            errorMessage: "",
            type: FormFieldType.SELECT,
            options: [
              { value: "SHOW", label: "Show" },
              { value: "HIDE", label: "Hide" },
            ],
            selectedValue: chartContent?.axis_minor_tick_show || "SHOW",
          },
        ],
      },
    },
  ];

  const runButton = (
    <Button
      key="run"
      label="Run"
      onClick={() => handleExecuteCell(cellId, true)}
      backgroundColor="green"
      size="sm"
      shape="rectangle"
      emphasis="secondary"
    />
  );
  return (
    <Form
      fields={fields}
      onSubmitCallback={handleSubmit}
      onCancelCallback={handleCancel}
      showCancelButton={false}
      submitButtonText="Save"
      rootStyleClass={CellEditorStyle.chartControlContainer}
      fieldsContainerStyleClass={CellEditorStyle.formElements}
      fieldStyleClass={CellEditorStyle.formField}
      additionalButtons={[runButton]}
      onChange={handleChange}
    />
  );
}
