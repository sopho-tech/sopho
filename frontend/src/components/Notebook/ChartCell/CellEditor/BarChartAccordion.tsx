import { Accordion } from "src/components/design-system";
import { Form } from "src/components/design-system";
import { Icon, Text, Flex } from "src/components/design-system";
import type { FormOptions } from "src/components/Notebook/ChartCell/CellEditor/types";
import {
  InfoTooltip,
  SORT_ORDER_OPTIONS,
  ORIENTATION_OPTIONS,
  VISIBILITY_OPTIONS,
  BAR_LAYOUT_OPTIONS,
} from "src/components/Notebook/ChartCell/CellEditor/utils";
import { SeriesFields } from "src/components/Notebook/ChartCell/CellEditor/SeriesFields";

export function BarChartAccordion({
  formOptions,
  accordionValues,
  onAccordionChange,
}: {
  formOptions: FormOptions;
  accordionValues: string[];
  onAccordionChange: (v: string[]) => void;
}) {
  const { xAxisColumnOptions } = formOptions;
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
          <Form.Field name="x_axis_title">
            <Form.Label>Title</Form.Label>
            <Form.Input placeholder="x-axis title" />
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
          <Form.Field name="y_axis_sort_by">
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Choose which series the chart is sorted by" />
              }
            >
              Sort By
            </Form.Label>
            <Form.Select
              options={formOptions.seriesSortOptions}
              groupName="Series"
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
          <Form.Field name="y_axis_title">
            <Form.Label>Title</Form.Label>
            <Form.Input placeholder="y-axis title" />
          </Form.Field>
          <SeriesFields formOptions={formOptions} />
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
          <Form.Field name="bar_layout">
            <Form.Label
              infoIconToolTipMessage={
                <InfoTooltip message="Grouped puts bars side by side; stacked puts them on top of each other" />
              }
            >
              Bar Layout
            </Form.Label>
            <Form.Select options={BAR_LAYOUT_OPTIONS} groupName="Bars" />
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
        </Accordion.Content>
      </Accordion.Item>
    </Accordion>
  );
}
