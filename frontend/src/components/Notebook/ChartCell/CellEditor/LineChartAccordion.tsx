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
} from "src/components/Notebook/ChartCell/CellEditor/utils";

export function LineChartAccordion({
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
          <Form.Field
            name="x_axis"
            required
            errorMessage="Please select the column for x-axis"
          >
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Select the column from the SQL query's output to be x-axis" />
              }
            >
              Column
            </Form.Label>
            <Form.Select options={xAxisColumnOptions} groupName="Column" />
          </Form.Field>
          <Form.Field name="x_axis_title" className={CellEditorStyle.formField}>
            <Form.Label>Title</Form.Label>
            <Form.Input
              placeholder="x-axis title"
              inputContainerClassName={CellEditorStyle.textInputContainer}
            />
          </Form.Field>
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
          <Form.Field
            name="y_axis"
            required
            errorMessage="Please select the column for y-axis"
          >
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Select the column from the SQL query's output for y-axis" />
              }
            >
              Column
            </Form.Label>
            <Form.Select options={yAxisColumnOptions} groupName="Column" />
          </Form.Field>
          <Form.Field
            name="y_axis_aggregate_function"
            required
            errorMessage="Please select the aggregate function for y-axis column"
          >
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Select the function to aggregate the y-axis column" />
              }
            >
              Aggregate by
            </Form.Label>
            <Form.Select
              options={yAxisAggregateFunctionsOptions}
              groupName="Aggregate"
            />
          </Form.Field>
          <Form.Field name="y_axis_sort_order">
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Specify how the chart should be sorted according to y-axis" />
              }
            >
              Sort Order
            </Form.Label>
            <Form.Select options={SORT_ORDER_OPTIONS} groupName="Sort" />
          </Form.Field>
          <Form.Field name="y_axis_title" className={CellEditorStyle.formField}>
            <Form.Label>Title</Form.Label>
            <Form.Input
              placeholder="y-axis title"
              inputContainerClassName={CellEditorStyle.textInputContainer}
            />
          </Form.Field>
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
          <Form.Field name="orientation">
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Choose how the chart should be layed out" />
              }
            >
              Orientation
            </Form.Label>
            <Form.Select options={ORIENTATION_OPTIONS} groupName="Layout" />
          </Form.Field>
          <Form.Field name="x_axis_tick_show">
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Choose whether to show the ticks on x-axis" />
              }
            >
              X-Axis Tick
            </Form.Label>
            <Form.Select options={VISIBILITY_OPTIONS} groupName="X-Axis" />
          </Form.Field>
          <Form.Field name="y_axis_tick_show">
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Choose whether to show the ticks on y-axis" />
              }
            >
              Y-Axis Tick
            </Form.Label>
            <Form.Select options={VISIBILITY_OPTIONS} groupName="Y-Axis" />
          </Form.Field>
          <Form.Field name="axis_minor_tick_show">
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Choose whether to show the minor ticks in x-axis" />
              }
            >
              Minor Tick
            </Form.Label>
            <Form.Select options={VISIBILITY_OPTIONS} groupName="Minor" />
          </Form.Field>
          <Form.Field name="show_dots">
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Choose whether to show dots on the line" />
              }
            >
              Show Dots
            </Form.Label>
            <Form.Select options={VISIBILITY_OPTIONS} groupName="Dots" />
          </Form.Field>
        </Accordion.Content>
      </Accordion.Item>
    </Accordion>
  );
}
