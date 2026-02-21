import { Form } from "src/components/design-system";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import type { FormOptions } from "src/components/Notebook/ChartCell/CellEditor/types";
import { InfoTooltip } from "src/components/Notebook/ChartCell/CellEditor/utils";

export function PieChartFields({ formOptions }: { formOptions: FormOptions }) {
  return (
    <>
      <Form.Field
        name="category"
        required
        errorMessage="Please select the category column"
      >
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Select the column to use as category labels" />
          }
          labelIconContainerStyleClass={CellEditorStyle.formLabel}
        >
          Category Name
        </Form.Label>
        <Form.Select
          options={formOptions.yAxisColumnOptions}
          groupName="Category"
        />
      </Form.Field>
      <Form.Field name="value" required errorMessage="Please select the value column">
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Select the column to use as values" />
          }
          labelIconContainerStyleClass={CellEditorStyle.formLabel}
        >
          Value Name
        </Form.Label>
        <Form.Select
          options={formOptions.xAxisColumnOptions}
          groupName="Value"
        />
      </Form.Field>
      <Form.Field
        name="aggregate_function"
        required
        errorMessage="Please select the aggregate function for y-axis column"
      >
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Select the function to aggregate the values" />
          }
          labelIconContainerStyleClass={CellEditorStyle.formLabel}
        >
          Aggregate by
        </Form.Label>
        <Form.Select
          options={formOptions.yAxisAggregateFunctionsOptions}
          groupName="Aggregate"
        />
      </Form.Field>
    </>
  );
}
