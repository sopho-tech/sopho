import { useUpdateCell, useCell } from "src/api/cell/queries";
import {
  CellDto,
  getChartContent,
  serializeChartContent,
} from "src/components/Notebook/Cell";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import {
  useSourceCellExecution,
  useFormOptions,
  useChartCellAutoLoad,
} from "src/components/Notebook/ChartCell/CellEditor/hooks";
import { useExecutionPhase } from "src/components/Notebook/Cell/useExecutionPhase";
import { Form } from "src/components/design-system";
import { getChartType } from "../../Cell/dto";
import {
  getDefaultValuesForChart,
  extractChartFormData,
  InfoTooltip,
} from "./utils";
import { BarChartAccordion } from "./BarChartAccordion";
import { LineChartAccordion } from "./LineChartAccordion";
import { PieChartFields } from "./PieChartFields";
import { MetricChartFields } from "./MetricChartFields";
import { ChartRunButton } from "./ChartRunButton";
import { ChartType } from "src/components/Chart";
import { useCallback, useMemo, useState } from "react";

/**
 * Component for editing a chart cell.
 *
 * The `initialChartContent` refers to the chart content of the cell from the backend, always in sync with it.
 * This differs from the chart content stored in the state store. The latter represents unsaved content being actively edited.
 * The latter is used for rendering the chart.
 *
 * @param cellId - ID of the chart cell to edit
 */
export function CellEditor({ cellId }: { cellId: string }) {
  const cellQuery = useCell(cellId);
  const initialChartContent = useMemo(
    () => (cellQuery.data ? getChartContent(cellQuery.data) : null),
    [cellQuery.data]
  );
  const { setSourceCellId, sourceCellId } = useSourceCellExecution(
    cellId,
    initialChartContent
  );
  useChartCellAutoLoad(cellId, initialChartContent);
  const { isRunning: isSourceRunning } = useExecutionPhase(sourceCellId ?? "");
  const updateCellMutation = useUpdateCell();
  const formOptions = useFormOptions(sourceCellId);
  const chartTypeFromContent = getChartType(initialChartContent);
  const [chartTypeOverride, setChartTypeOverride] = useState<ChartType | null>(null);
  const chartType = chartTypeOverride ?? chartTypeFromContent;
  const [accordionValues, setAccordionValues] = useState<string[]>([]);
  const defaultValues = getDefaultValuesForChart(
    chartType,
    initialChartContent
  );

  const handleSubmit = useCallback(
    (formData: FormData) => {
      if (!cellQuery.data) throw Error("Cell query data is empty");
      if (!chartType) throw Error("Cell chart type is empty");
      const content = extractChartFormData(chartType, formData);
      const cellDto: CellDto = {
        ...cellQuery.data,
        content: serializeChartContent(content),
      };
      updateCellMutation.mutate({ cellId, payload: cellDto });
    },
    [cellId, cellQuery.data, chartType, updateCellMutation]
  );

  const handleChange = useCallback(
    (_: FormData, fieldName: string, value: string) => {
      if (fieldName === "cell_id") {
        setSourceCellId(value);
      } else if (fieldName === "chart_type") {
        setChartTypeOverride(ChartType[value as keyof typeof ChartType] ?? null);
      }
    },
    [setSourceCellId]
  );

  return (
    <Form
      defaultValues={defaultValues}
      onSubmit={handleSubmit}
      onChange={handleChange}
      readonly={isSourceRunning || formOptions.isAwaitingSourceColumns}
      className={CellEditorStyle.chartControlContainer}
    >
      <Form.ErrorBanner />
      <Form.Fields className={CellEditorStyle.formElements}>
        <Form.Field
          name="cell_id"
          required
          errorMessage="Please select the source cell"
        >
          <Form.Label
            infoIconToolTipMessage={
              <InfoTooltip message="Select the source SQL Cell whose query you want to visualize" />
            }
            labelIconContainerStyleClass={CellEditorStyle.formLabel}
          >
            Source Cell
          </Form.Label>
          <Form.Select options={formOptions.cellOptions} groupName="Source" />
        </Form.Field>
        <Form.Field
          name="chart_type"
          required
          errorMessage="Please select the type of chart"
        >
          <Form.Label
            infoIconToolTipMessage={
              <InfoTooltip message="Select the type of chart you want to visualize data as" />
            }
            labelIconContainerStyleClass={CellEditorStyle.formLabel}
          >
            Chart Type
          </Form.Label>
          <Form.Select options={formOptions.chartOptions} groupName="Type" />
        </Form.Field>
        {chartType === ChartType.BAR && (
          <BarChartAccordion
            formOptions={formOptions}
            accordionValues={accordionValues}
            onAccordionChange={setAccordionValues}
          />
        )}
        {chartType === ChartType.LINE && (
          <LineChartAccordion
            formOptions={formOptions}
            accordionValues={accordionValues}
            onAccordionChange={setAccordionValues}
          />
        )}
        {chartType === ChartType.PIE && (
          <PieChartFields formOptions={formOptions} />
        )}
        {chartType === ChartType.METRIC && <MetricChartFields />}
      </Form.Fields>
      <Form.Actions className={CellEditorStyle.actionsButtonContainer}>
        <Form.Submit label="Save" />
        <ChartRunButton cellId={cellId} chartType={chartType} />
      </Form.Actions>
    </Form>
  );
}
