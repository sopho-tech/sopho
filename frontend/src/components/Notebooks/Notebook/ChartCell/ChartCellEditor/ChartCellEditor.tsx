import ChartCellEditorStyles from "src/components/Notebooks/Notebook/ChartCell/ChartCellEditor/ChartCellEditor.module.css";
import {
  SophoForm,
  SophoFormElement,
  SophoFormElementType,
} from "src/components/SophoForm/SophoForm";
import { useNotebookStore } from "src/components/Notebooks/store";
import { useNotebook } from "src/api/notebook/queries";
import { CellType } from "src/components/Notebooks/Notebook/Cell/dto";

import {
  HorizontalProgressBar,
  ProgressStep,
} from "src/components/HorizontalProgressBar";
import { useState, useEffect } from "react";
import { useUpdateCell, useCell } from "src/api/cell/queries";

export enum ChartType {
  BAR = "BAR",
  LINE = "LINE",
  PIE = "PIE",
  SCATTER = "SCATTER",
}

enum FormPage {
  FIRST = "FIRST",
  SECOND = "SECOND",
  THIRD = "THIRD",
}

export function ChartCellEditor({ cellId }: { cellId: string }) {
  const { currentNotebookId } = useNotebookStore();
  const notebookQuery = useNotebook(currentNotebookId);
  const [formPage, setFormPage] = useState(FormPage.FIRST);
  const updateCellMutation = useUpdateCell();
  const cellQuery = useCell(cellId);

  const currentCell = cellQuery.data;

  const [formData, setFormData] = useState<any>({});

  useEffect(() => {
    if (currentCell?.content) {
      try {
        const parsedContent = JSON.parse(currentCell.content);
        const newFormData = {
          cell: parsedContent.cell_id || "",
          chart_type: parsedContent.chart_type || "",
          x_axis: parsedContent.x_axis || "",
          y_axis: parsedContent.y_axis || "",
        };
        setFormData(newFormData);
      } catch (error) {
        setFormData({});
      }
    }
  }, [currentCell]);

  const progressSteps: ProgressStep[] = [
    {
      id: FormPage.FIRST,
      label: "Data Source ",
    },
    {
      id: FormPage.SECOND,
      label: "Chart Type",
    },
    {
      id: FormPage.THIRD,
      label: "Chart Configurations",
    },
  ];

  function handleStepClick(stepId: string) {
    setFormPage(stepId as FormPage);
  }

  function onSubmitCallback(formDataParam: FormData) {
    if (!currentCell) return;

    if (formPage == FormPage.FIRST) {
      const newFormData = {
        ...formData,
        cell: formDataParam.get("cell"),
      };
      setFormData(newFormData);
      setFormPage(FormPage.SECOND);
    } else if (formPage == FormPage.SECOND) {
      const newFormData = {
        ...formData,
        chart_type: formDataParam.get("chart_type"),
      };
      setFormData(newFormData);
      setFormPage(FormPage.THIRD);
    } else {
      const updatedCell = {
        ...currentCell,
        content: JSON.stringify({
          x_axis: formDataParam.get("x_axis") as string,
          y_axis: formDataParam.get("y_axis") as string,
          chart_type: formData.chart_type,
          cell_id: formData.cell,
        }),
      };
      updateCellMutation.mutate({ cellId, payload: updatedCell });
    }
  }

  function onCancelCallback() {}

  function onXAxisChange(event: React.ChangeEvent<HTMLInputElement>) {
    setFormData({
      ...formData,
      x_axis: event.target.value || "",
    });
  }

  function onYAxisChange(event: React.ChangeEvent<HTMLInputElement>) {
    setFormData({
      ...formData,
      y_axis: event.target.value || "",
    });
  }

  const cellOptions =
    notebookQuery.data?.cells
      .filter((cell) => cell.cell_type === CellType.SQL)
      .map((cell) => ({
        value: cell.id,
        label: cell.name || `Cell ${cell.display_order}`,
      })) || [];

  const chartTypeOptions = Object.values(ChartType).map((type) => ({
    value: type,
    label: type.charAt(0) + type.slice(1).toLowerCase(),
  }));

  function renderForm() {
    if (cellQuery.isLoading || !currentCell) {
      return <div>Loading...</div>;
    }

    if (formPage == FormPage.FIRST) {
      const formElements: SophoFormElement[] = [
        {
          key: "cell",
          name: "Source Cell",
          required: true,
          message: "Please select the source cell",
          type: SophoFormElementType.SELECT,
          options: cellOptions,
          selectedValue: formData?.cell,
        },
      ];
      return (
        <SophoForm
          formElements={formElements}
          onSubmitCallback={onSubmitCallback}
          onCancelCallback={onCancelCallback}
          submitButtonText="Next"
          showCancelButton={false}
        />
      );
    } else if (formPage == FormPage.SECOND) {
      const formElements: SophoFormElement[] = [
        {
          key: "chart_type",
          name: "Chart Type",
          required: true,
          message: "Please fill the chart type",
          type: SophoFormElementType.SELECT,
          options: chartTypeOptions,
          selectedValue: formData?.chart_type,
        },
      ];
      return (
        <SophoForm
          formElements={formElements}
          onSubmitCallback={onSubmitCallback}
          onCancelCallback={onCancelCallback}
          submitButtonText="Next"
        />
      );
    } else if (formPage == FormPage.THIRD) {
      const formElements: SophoFormElement[] = [
        {
          key: "x_axis",
          name: "X-Axis",
          required: true,
          message: "Please input the column for x-axis",
          type: SophoFormElementType.INPUT,
          initialValue: formData?.x_axis || "",
          onChange: onXAxisChange,
        },
        {
          key: "y_axis",
          name: "Y-Axis",
          required: true,
          message: "Please input the column for y-axis",
          type: SophoFormElementType.INPUT,
          initialValue: formData?.y_axis || "",
          onChange: onYAxisChange,
        },
      ];
      return (
        <SophoForm
          formElements={formElements}
          onSubmitCallback={onSubmitCallback}
          onCancelCallback={onCancelCallback}
          submitButtonText="Save"
          showCancelButton={false}
        />
      );
    }
  }
  return (
    <div className={ChartCellEditorStyles.container}>
      <HorizontalProgressBar
        steps={progressSteps}
        currentStepId={formPage}
        onStepClick={handleStepClick}
      />
      {renderForm()}
    </div>
  );
}
