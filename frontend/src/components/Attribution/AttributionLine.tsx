import { AttributionDto } from "src/api/dto";
import { Avatar, Flex, Icon, Text } from "src/components/design-system";
import {
  CREATED_BY_LABEL,
  EDITED_BY_LABEL,
  LAST_EDITED_LABEL,
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
    <Flex direction="row" alignItems="center" gap="lg" flexWrap="wrap">
      <Flex direction="row" alignItems="center" gap="2xs">
        <Avatar name={describeActor(createdBy)} size="sm" />
        <Text fontSize="xs" color="subtle">
          {CREATED_BY_LABEL}
        </Text>
        <Text fontSize="xs" color="darkGrey">
          {describeActor(createdBy)}
        </Text>
      </Flex>
      <Flex direction="row" alignItems="center" gap="2xs">
        <Icon type="clock" color="grey" size="sm" />
        <Text fontSize="xs" color="subtle">
          {LAST_EDITED_LABEL}
        </Text>
        <Text fontSize="xs" color="darkGrey">
          {lastEdited}
        </Text>
        <Text fontSize="xs" color="subtle">
          {EDITED_BY_LABEL}
        </Text>
        <Text fontSize="xs" color="darkGrey">
          {describeActor(modifiedBy)}
        </Text>
      </Flex>
    </Flex>
  );
}
