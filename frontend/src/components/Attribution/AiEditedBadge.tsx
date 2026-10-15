import { ActorKind, AttributionDto } from "src/api/dto";
import { Badge, Flex, ToolTip } from "src/components/design-system";
import {
  AI_BADGE_LABEL,
  LAST_EDITED_BY_LABEL,
} from "src/components/Attribution/constants";
import { describeActor } from "src/components/Attribution/describeActor";

type AiEditedBadgeProps = {
  modifiedBy: AttributionDto;
};

export function AiEditedBadge({ modifiedBy }: AiEditedBadgeProps) {
  if (modifiedBy.actor !== ActorKind.AI) {
    return null;
  }
  return (
    <ToolTip
      messageElement={`${LAST_EDITED_BY_LABEL} ${describeActor(modifiedBy)}`}
      tooltipSide="top"
    >
      <Flex alignItems="center">
        <Badge variant="blue">{AI_BADGE_LABEL}</Badge>
      </Flex>
    </ToolTip>
  );
}
