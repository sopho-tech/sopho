import {
  FormField,
  FormFieldType,
  ValidationTrigger,
} from "src/components/design-system/Form";
import { useUpdateCell, useCell } from "src/api/cell/queries";
import {
  CellDto,
  getChartContent,
  serializeChartContent,
} from "src/components/Notebooks/Notebook/Cell";
import CellEditorStyle from "src/components/Notebooks/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import { Button } from "src/components/design-system/Button";
import {
  useSourceCellExecution,
  useFormOptions,
} from "src/components/Notebooks/Notebook/ChartCell/CellEditor/hooks";
import { Form, Icon, Text, Flex } from "src/components/design-system";
import { Accordion } from "src/components/Accordion";

export function CellEditor({ cellId }: { cellId: string }) {
  const cellQuery = useCell(cellId);
  const updateCellMutation = useUpdateCell();
  const chartContent = cellQuery.data ? getChartContent(cellQuery.data) : null;
  const { setSourceCellId, sourceCellOutput, executeSourceCell } =
    useSourceCellExecution(cellId, chartContent);
  const { cellOptions, chartOptions, columnOptions } =
    useFormOptions(sourceCellOutput);

  function handleSubmit(formData: FormData) {
    if (!cellQuery.data) throw Error("Cell query data is empty");
    const chartContent = {
      x_axis: (formData.get("x_axis") as string) || "",
      y_axis: (formData.get("y_axis") as string) || "",
      chart_type: (formData.get("chart_type") as string) || undefined,
      cell_id: (formData.get("cell_id") as string) || undefined,
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
            options: columnOptions,
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
            options: columnOptions,
            selectedValue: chartContent?.y_axis,
          },
        ],
      },
    },
  ];

  const runButton = (
    <Button
      key="run"
      label="Run"
      onClick={() => executeSourceCell()}
      backgroundColor="green"
      size="sm"
      shape="rectangle"
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
