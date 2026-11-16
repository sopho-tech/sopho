import { useCallback } from "react";
import { useLocation, useNavigate, useParams } from "react-router";
import { usePlatformUsageOfConversationQuestions } from "src/api/platform_ai_usage";
import { Flex } from "src/components/design-system";
import { ConversationQuestions } from "src/components/Settings/AIUsage/UsageConversationQuestions";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { useUsageRange } from "src/components/Settings/AIUsage/hooks";
import { APP_ROUTES } from "src/constants/app_routes";

export function PlatformUsageConversationQuestions() {
  const { usageTenantId, conversationId } = useParams();
  if (!usageTenantId || !conversationId) {
    throw new Error(
      "The tenant or conversation id is missing from the address",
    );
  }
  const navigate = useNavigate();
  const location = useLocation();
  const { lastDayCount } = useUsageRange();
  const conversationQuestions = usePlatformUsageOfConversationQuestions(
    lastDayCount,
    usageTenantId,
    conversationId,
  );
  const handleBack = useCallback(
    () =>
      void navigate({
        pathname: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_BY_CONVERSATION,
        search: location.search,
      }),
    [navigate, location.search],
  );

  return (
    <Flex direction="column">
      {conversationQuestions.error && (
        <UsageFailure failure={conversationQuestions.error} />
      )}
      {conversationQuestions.isPending && <UsageLoading />}
      {conversationQuestions.data && (
        <ConversationQuestions
          conversationQuestions={conversationQuestions.data}
          onBack={handleBack}
        />
      )}
    </Flex>
  );
}
