import { useCallback } from "react";
import { CreatedByFilter } from "src/api/dto";
import { SegmentedControl } from "src/components/design-system";
import {
  ALL_ROWS_FILTER_LABEL,
  ALL_ROWS_FILTER_VALUE,
  CREATED_BY_ME_FILTER_LABEL,
} from "src/components/Attribution/constants";

const CREATED_BY_FILTER_OPTIONS = [
  { label: ALL_ROWS_FILTER_LABEL, value: ALL_ROWS_FILTER_VALUE },
  { label: CREATED_BY_ME_FILTER_LABEL, value: CreatedByFilter.ME },
];

type CreatedByFilterControlProps = {
  value: CreatedByFilter | null;
  onValueChange: (value: CreatedByFilter | null) => void;
};

export function CreatedByFilterControl({
  value,
  onValueChange,
}: CreatedByFilterControlProps) {
  const handleValueChange = useCallback(
    (selected: string) =>
      onValueChange(
        selected === CreatedByFilter.ME ? CreatedByFilter.ME : null,
      ),
    [onValueChange],
  );
  return (
    <SegmentedControl
      size="sm"
      options={CREATED_BY_FILTER_OPTIONS}
      value={value ?? ALL_ROWS_FILTER_VALUE}
      onValueChange={handleValueChange}
    />
  );
}
