import { AttributionDto } from "src/api/dto";
import { Text } from "src/components/design-system";
import {
  CREATED_BY_LABEL,
  LAST_EDITED_BY_LABEL,
} from "src/components/Attribution/constants";
import { describeActor } from "src/components/Attribution/describeActor";
import { formatRelativeTime } from "src/utils/timestamp_utils";

type AttributionLineProps = {
  createdBy: AttributionDto;
  modifiedBy: AttributionDto;
  updatedAt: string | null;
};

export function AttributionLine({
  createdBy,
  modifiedBy,
  updatedAt,
}: AttributionLineProps) {
  const lastEdited = formatRelativeTime(updatedAt);
  if (lastEdited === null) {
    throw new Error("An attribution line needs the time of the last edit");
  }
  return (
    <Text fontSize="xs" color="subtle">
      {`${CREATED_BY_LABEL} ${describeActor(createdBy)} · ${LAST_EDITED_BY_LABEL} ${describeActor(modifiedBy)}, ${lastEdited}`}
    </Text>
  );
}
