import { useUpdateCell, useCell } from "src/api/cell/queries";
import {
  CellDto,
  getChartContent,
  serializeChartContent,
  useHandleExecuteCell,
} from "src/components/Notebook/Cell";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import { Button } from "src/components/design-system/Button";
import {
  useSourceCellExecution,
  useFormOptions,
} from "src/components/Notebook/ChartCell/CellEditor/hooks";
import { Form } from "src/components/design-system";
import { useCallback, useState, useEffect } from "react";
import { ChartType } from "src/components/Chart";
import { getChartType } from "../../Cell/dto";
import { getFormFieldsByChartType, extractChartFormData } from "./utils";

export function CellEditor({ cellId }: { cellId: string }) {
  const cellQuery = useCell(cellId);
  const updateCellMutation = useUpdateCell();
  const chartContent = cellQuery.data ? getChartContent(cellQuery.data) : null;
  const { setSourceCellId, sourceCellOutput } = useSourceCellExecution(
    cellId,
    chartContent
  );
  const handleExecuteCell = useHandleExecuteCell();
  const formOptions = useFormOptions(sourceCellOutput);
  const initialChartType = getChartType(chartContent);
  const [chartType, setChartType] = useState<ChartType | null>(
    initialChartType
  );

  useEffect(() => {
    if (initialChartType !== null) {
      setChartType(initialChartType);
    }
  }, [initialChartType]);

  const handleSubmit = useCallback(
    (formData: FormData) => {
      if (!cellQuery.data) throw Error("Cell query data is empty");

      const chartContent = extractChartFormData(chartType, formData);
      const cellDto: CellDto = {
        ...cellQuery.data,
        content: serializeChartContent(chartContent),
      };
      updateCellMutation.mutate({ cellId, payload: cellDto });
    },
    [cellQuery.data, chartType, cellId, updateCellMutation]
  );

  const handleChange = useCallback(
    (_: FormData, fieldName: string, value: string) => {
      if (fieldName === "cell_id") {
        setSourceCellId(value);
      } else if (fieldName === "chart_type") {
        setChartType(ChartType[value as keyof typeof ChartType] ?? null);
      }
    },
    [setSourceCellId]
  );

  const getFormFields = useCallback(
    () => getFormFieldsByChartType(chartType, chartContent, formOptions),
    [chartType, chartContent, formOptions]
  );

  const runButton = (
    <Button
      key="run"
      label="Run"
      onClick={() => handleExecuteCell(cellId, true)}
      backgroundColor="transparent"
      size="sm"
      shape="rectangle"
      emphasis="secondary"
    />
  );

  return (
    <Form
      fields={getFormFields()}
      onSubmitCallback={handleSubmit}
      onCancelCallback={() => {}}
      showCancelButton={false}
      submitButtonText="Save"
      rootStyleClass={CellEditorStyle.chartControlContainer}
      fieldsContainerStyleClass={CellEditorStyle.formElements}
      fieldStyleClass={CellEditorStyle.formField}
      labelStyleClass={CellEditorStyle.formLabel}
      additionalButtons={[runButton]}
      onChange={handleChange}
    />
  );
}
