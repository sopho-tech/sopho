import {
  SophoForm,
  SophoFormElement,
  SophoFormElementType,
} from "src/components/SophoForm";
import { useUpdateCell, useCell } from "src/api/cell/queries";
import {
  CellDto,
  getChartContent,
  serializeChartContent,
} from "src/components/Notebooks/Notebook/Cell";
import CellEditorStyle from "src/components/Notebooks/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import { NewAssetButton } from "src/components/NewAssetButton";
import {
  useSourceCellExecution,
  useFormOptions,
} from "src/components/Notebooks/Notebook/ChartCell/CellEditor/hooks";

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

  const formElements: SophoFormElement[] = [
    {
      key: "cell_id",
      name: "Source Cell",
      required: true,
      error_message: "Please select the source cell",
      type: SophoFormElementType.SELECT,
      options: cellOptions,
      selectedValue: chartContent?.cell_id,
    },
    {
      key: "chart_type",
      name: "Chart Type",
      required: true,
      error_message: "Please select the type of chart",
      type: SophoFormElementType.SELECT,
      options: chartOptions,
      selectedValue: chartContent?.chart_type,
    },
    {
      key: "x_axis_settings",
      name: (
        <div className={CellEditorStyle.accordionTriggerHeading}>
          <span
            className={`material-symbols-outlined ${CellEditorStyle.accordionTriggerIcon}`}
          >
            swap_horiz
          </span>
          <span>X-axis</span>
        </div>
      ),
      required: false,
      error_message: "",
      type: SophoFormElementType.COLLAPSIBLE,
      collapsibleConfig: {
        formElements: [
          {
            key: "x_axis",
            name: "Column",
            required: true,
            error_message: "Please select the column for x-axis",
            type: SophoFormElementType.SELECT,
            options: columnOptions,
            selectedValue: chartContent?.x_axis,
          },
        ],
      },
    },
    {
      key: "y_axis_settings",
      name: (
        <div className={CellEditorStyle.accordionTriggerHeading}>
          <span
            className={`material-symbols-outlined ${CellEditorStyle.accordionTriggerIcon}`}
          >
            swap_vert
          </span>
          <span>Y-axis</span>
        </div>
      ),
      required: false,
      error_message: "",
      type: SophoFormElementType.COLLAPSIBLE,
      collapsibleConfig: {
        formElements: [
          {
            key: "y_axis",
            name: "Column",
            required: true,
            error_message: "Please select the column for y-axis",
            type: SophoFormElementType.SELECT,
            options: columnOptions,
            selectedValue: chartContent?.y_axis,
          },
        ],
      },
    },
  ];

  const runButton = (
    <NewAssetButton
      key="run"
      buttonText="Run"
      className={CellEditorStyle.runButton}
      onClick={executeSourceCell}
    />
  );
  return (
    <SophoForm
      formElements={formElements}
      onSubmitCallback={handleSubmit}
      onCancelCallback={handleCancel}
      showCancelButton={false}
      submitButtonText="Save"
      formRootStyleClass={CellEditorStyle.chartControlContainer}
      formElementsStyleClass={CellEditorStyle.formElements}
      formFieldStyleClass={CellEditorStyle.formField}
      additionalButtons={[runButton]}
      onChange={handleChange}
    />
  );
}
