import { useCallback } from "react";
import {
  Button,
  Flex,
  Form,
  Heading,
  IconButton,
} from "src/components/design-system";
import type { FormOptions } from "src/components/Notebook/ChartCell/CellEditor/types";
import {
  InfoTooltip,
  SERIES_LIMIT,
  nextColorIndex,
} from "src/components/Notebook/ChartCell/CellEditor/utils";

const newSeries = (items: unknown[]) => ({
  column: "",
  aggregate_function: "",
  label: "",
  color_index: nextColorIndex(items as { color_index?: number }[]),
});

type SeriesRowProps = {
  index: number;
  removable: boolean;
  formOptions: FormOptions;
  onRemove: (index: number) => void;
};

function SeriesRow({
  index,
  removable,
  formOptions,
  onRemove,
}: SeriesRowProps) {
  const { yAxisColumnOptions, yAxisAggregateFunctionsOptions } = formOptions;
  const handleRemove = useCallback(() => onRemove(index), [index, onRemove]);

  return (
    <Flex
      direction="column"
      gap="xs"
      border="default"
      borderRadius="lg"
      paddingX="sm"
      paddingY="sm"
    >
      <Flex direction="row" justifyContent="space-between" alignItems="center">
        <Heading
          accessbilityLevel={4}
          size="sm"
          weight="semibold"
          textColor="subtle"
        >
          {`Series ${index + 1}`}
        </Heading>
        {removable && (
          <IconButton
            type="delete"
            backgroundColor="default"
            iconColor="grey"
            iconSize="sm"
            onClick={handleRemove}
            aria-label={`Remove series ${index + 1}`}
          />
        )}
      </Flex>
      <Form.Field
        name={`series[${index}].column`}
        required
        errorMessage="Please select the column for this series"
      >
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Select the column from the SQL query's output to plot" />
          }
        >
          Column
        </Form.Label>
        <Form.Select options={yAxisColumnOptions} groupName="Column" />
      </Form.Field>
      <Form.Field
        name={`series[${index}].aggregate_function`}
        required
        errorMessage="Please select the aggregate function for this series"
      >
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Select the function to aggregate this column" />
          }
        >
          Aggregate by
        </Form.Label>
        <Form.Select
          options={yAxisAggregateFunctionsOptions}
          groupName="Aggregate"
        />
      </Form.Field>
      <Form.Field name={`series[${index}].label`}>
        <Form.Label
          infoIconToolTipMessage={
            <InfoTooltip message="Name shown in the legend. Defaults to the column name" />
          }
        >
          Label
        </Form.Label>
        <Form.Input placeholder="legend label" />
      </Form.Field>
    </Flex>
  );
}

export function SeriesFields({ formOptions }: { formOptions: FormOptions }) {
  return (
    <Form.FieldArray name="series" newItem={newSeries}>
      {({ items, push, remove }) => (
        <Flex direction="column" gap="sm">
          {items.map((_, index) => (
            <SeriesRow
              key={index}
              index={index}
              removable={items.length > 1}
              formOptions={formOptions}
              onRemove={remove}
            />
          ))}
          <Flex direction="row" justifyContent="flex-end">
            <Button
              label="Add Series"
              onClick={push}
              disabled={items.length >= SERIES_LIMIT}
              backgroundColor="accent"
              size="sm"
              shape="rectangle"
            />
          </Flex>
        </Flex>
      )}
    </Form.FieldArray>
  );
}
