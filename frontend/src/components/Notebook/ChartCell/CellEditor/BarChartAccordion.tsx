import { Accordion } from "src/components/design-system";
import { Form } from "src/components/design-system";
import { Icon, Text, Flex } from "src/components/design-system";
import CellEditorStyle from "src/components/Notebook/ChartCell/CellEditor/CellEditor.module.css";
import type { FormOptions } from "src/components/Notebook/ChartCell/CellEditor/types";
import {
  InfoTooltip,
  SORT_ORDER_OPTIONS,
  ORIENTATION_OPTIONS,
  VISIBILITY_OPTIONS,
  formLabel,
} from "src/components/Notebook/ChartCell/CellEditor/utils";

export function BarChartAccordion({
  formOptions,
  accordionValues,
  onAccordionChange,
}: {
  formOptions: FormOptions;
  accordionValues: string[];
  onAccordionChange: (v: string[]) => void;
}) {
  const {
    xAxisColumnOptions,
    yAxisColumnOptions,
    yAxisAggregateFunctionsOptions,
  } = formOptions;
  const labelProps = formLabel(CellEditorStyle.formLabel);
  return (
    <Accordion value={accordionValues} onValueChange={onAccordionChange}>
      <Accordion.Item value="x_axis_settings">
        <Accordion.Trigger>
          <Flex direction="row" gap="2xs" justifyContent="center">
            <Icon type="swap_horiz" color="default" />
            <Text fontSize="sm">X-axis</Text>
          </Flex>
        </Accordion.Trigger>
        <Accordion.Content>
          <Form.Select
            name="x_axis"
            label="Column"
            options={xAxisColumnOptions}
            required
            errorMessage="Please select the column for x-axis"
            infoIconToolTipMessage={
              <InfoTooltip message="Select the column from the SQL query's output to be x-axis" />
            }
            {...labelProps}
          />
          <Form.Input
            name="x_axis_title"
            label="Title"
            placeholder="Enter x-axis title"
            labelClassName={CellEditorStyle.formLabel}
            className={CellEditorStyle.formField}
            inputContainerClassName={CellEditorStyle.textInputContainer}
          />
        </Accordion.Content>
      </Accordion.Item>
      <Accordion.Item value="y_axis_settings">
        <Accordion.Trigger>
          <Flex direction="row" gap="2xs" justifyContent="center">
            <Icon type="swap_vert" color="default" />
            <Text fontSize="sm">Y-axis</Text>
          </Flex>
        </Accordion.Trigger>
        <Accordion.Content>
          <Form.Select
            name="y_axis"
            label="Column"
            options={yAxisColumnOptions}
            required
            errorMessage="Please select the column for y-axis"
            infoIconToolTipMessage={
              <InfoTooltip message="Select the column from the SQL query's output for y-axis" />
            }
            {...labelProps}
          />
          <Form.Select
            name="y_axis_aggregate_function"
            label="Aggregate by"
            options={yAxisAggregateFunctionsOptions}
            required
            errorMessage="Please select the aggregate function for y-axis column"
            infoIconToolTipMessage={
              <InfoTooltip message="Select the function to aggregate the y-axis column" />
            }
            {...labelProps}
          />
          <Form.Select
            name="y_axis_sort_order"
            label="Sort Order"
            options={SORT_ORDER_OPTIONS}
            infoIconToolTipMessage={
              <InfoTooltip message="Specify how the chart should be sorted according to y-axis" />
            }
            {...labelProps}
          />
          <Form.Input
            name="y_axis_title"
            label="Title"
            placeholder="Enter y-axis title"
            labelClassName={CellEditorStyle.formLabel}
            className={CellEditorStyle.formField}
            inputContainerClassName={CellEditorStyle.textInputContainer}
          />
        </Accordion.Content>
      </Accordion.Item>
      <Accordion.Item value="display_settings">
        <Accordion.Trigger>
          <Flex direction="row" gap="xs" alignItems="center">
            <Icon type="settings" color="default" />
            <Text fontSize="sm">Display</Text>
          </Flex>
        </Accordion.Trigger>
        <Accordion.Content>
          <Form.Select
            name="orientation"
            label="Orientation"
            options={ORIENTATION_OPTIONS}
            infoIconToolTipMessage={
              <InfoTooltip message="Choose how the chart should be layed out" />
            }
            {...labelProps}
          />
          <Form.Select
            name="x_axis_tick_show"
            label="X-Axis Tick"
            options={VISIBILITY_OPTIONS}
            infoIconToolTipMessage={
              <InfoTooltip message="Choose whether to show the ticks on x-axis" />
            }
            {...labelProps}
          />
          <Form.Select
            name="y_axis_tick_show"
            label="Y-Axis Tick"
            options={VISIBILITY_OPTIONS}
            infoIconToolTipMessage={
              <InfoTooltip message="Choose whether to show the ticks on y-axis" />
            }
            {...labelProps}
          />
          <Form.Select
            name="axis_minor_tick_show"
            label="Minor Tick"
            options={VISIBILITY_OPTIONS}
            infoIconToolTipMessage={
              <InfoTooltip message="Choose whether to show the minor ticks in x-axis" />
            }
            {...labelProps}
          />
        </Accordion.Content>
      </Accordion.Item>
    </Accordion>
  );
}
