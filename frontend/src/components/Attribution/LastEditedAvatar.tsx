import { ActorKind, AttributionDto } from "src/api/dto";
import { Avatar, Flex, ToolTip } from "src/components/design-system";
import {
  EDITED_ON_LABEL,
  SYSTEM_ACTOR_NAME,
  USING_AI_LABEL,
} from "src/components/Attribution/constants";
import { formatTimestamp } from "src/utils/timestamp_utils";

type LastEditedAvatarProps = {
  modifiedBy: AttributionDto;
  updatedAt: string;
};

function editorName(modifiedBy: AttributionDto): string {
  if (modifiedBy.actor === ActorKind.SYSTEM) {
    return SYSTEM_ACTOR_NAME;
  }
  if (!modifiedBy.user) {
    throw new Error(`A ${modifiedBy.actor} attribution has no user`);
  }
  return modifiedBy.user.full_name;
}

export function LastEditedAvatar({
  modifiedBy,
  updatedAt,
}: LastEditedAvatarProps) {
  const name = editorName(modifiedBy);
  const edited = `${name} ${EDITED_ON_LABEL} ${formatTimestamp(updatedAt)}`;
  const details =
    modifiedBy.actor === ActorKind.AI ? `${edited} ${USING_AI_LABEL}` : edited;
  return (
    <ToolTip messageElement={details} tooltipSide="top">
      <Flex alignItems="center" role="img" aria-label={details}>
        <Avatar name={name} size="sm" />
      </Flex>
    </ToolTip>
  );
}
