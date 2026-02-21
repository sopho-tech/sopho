import { Form } from "src/components/design-system";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import {
  InfoTooltip,
  METRIC_FORMAT_OPTIONS,
} from "src/components/Notebook/ChartCell/CellEditor/utils";

export function MetricChartFields() {
  return (
    <>
      <Form.Field name="format">
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Format of the metric value: Default (custom suffix), Percentage (%), or Currency ($)" />
          }
          labelIconContainerStyleClass={CellEditorStyle.formLabel}
        >
          Format
        </Form.Label>
        <Form.Select options={METRIC_FORMAT_OPTIONS} groupName="Format" />
      </Form.Field>
      <Form.Field name="decimal_precision">
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Number of decimal places to display for the metric value" />
          }
          labelIconContainerStyleClass={CellEditorStyle.formLabel}
        >
          Decimal Precision
        </Form.Label>
        <Form.Input placeholder="e.g. 2" />
      </Form.Field>
      <Form.Field name="suffix">
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Optional text displayed after the metric value (e.g. %, $, km)" />
          }
          labelIconContainerStyleClass={CellEditorStyle.formLabel}
        >
          Suffix
        </Form.Label>
        <Form.Input placeholder="e.g. %, $, km" />
      </Form.Field>
    </>
  );
}
