import { ActorKind, AttributionDto } from "src/api/dto";
import {
  AI_ACTOR_PREFIX,
  SYSTEM_ACTOR_NAME,
} from "src/components/Attribution/constants";

export function describeActor(attribution: AttributionDto): string {
  if (attribution.actor === ActorKind.SYSTEM) {
    return SYSTEM_ACTOR_NAME;
  }
  if (!attribution.user) {
    throw new Error(`A ${attribution.actor} attribution has no user`);
  }
  if (attribution.actor === ActorKind.AI) {
    return `${AI_ACTOR_PREFIX} ${attribution.user.full_name}`;
  }
  return attribution.user.full_name;
}
