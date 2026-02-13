import { Form } from "src/components/design-system";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import type { FormOptions } from "src/components/Notebook/ChartCell/CellEditor/types";
import { InfoTooltip, formLabel } from "src/components/Notebook/ChartCell/CellEditor/utils";

export function PieChartFields({ formOptions }: { formOptions: FormOptions }) {
  return (
    <>
      <Form.Select
        name="category"
        label="Category Name"
        options={formOptions.yAxisColumnOptions}
        required
        errorMessage="Please select the category column"
        infoIconToolTipMessage={
          <InfoTooltip message="Select the column to use as category labels" />
        }
        {...formLabel(CellEditorStyle.formLabel)}
      />
      <Form.Select
        name="value"
        label="Value Name"
        options={formOptions.xAxisColumnOptions}
        required
        errorMessage="Please select the value column"
        infoIconToolTipMessage={
          <InfoTooltip message="Select the column to use as values" />
        }
        {...formLabel(CellEditorStyle.formLabel)}
      />
      <Form.Select
        name="aggregate_function"
        label="Aggregate by"
        options={formOptions.yAxisAggregateFunctionsOptions}
        required
        errorMessage="Please select the aggregate function for y-axis column"
        infoIconToolTipMessage={
          <InfoTooltip message="Select the function to aggregate the values" />
        }
        {...formLabel(CellEditorStyle.formLabel)}
      />
    </>
  );
}
