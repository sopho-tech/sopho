import { useCallback, useMemo } from "react";
import { useSearchParams } from "react-router";
import { UsageRangeParams } from "src/api/ai_usage";
import {
  CONVERSATION_PAGE_SEARCH_PARAMETER,
  DEFAULT_RANGE_OPTION,
  FIRST_CONVERSATION_PAGE,
  RANGE_SEARCH_PARAMETER,
} from "src/components/Settings/AIUsage/constants";

export function useUsageRange(): UsageRangeParams {
  const [searchParams] = useSearchParams();
  const lastDayCount =
    searchParams.get(RANGE_SEARCH_PARAMETER) ?? DEFAULT_RANGE_OPTION;
  return useMemo(() => ({ lastDayCount }), [lastDayCount]);
}

export function useChangeUsageRange(): (rangeOption: string) => void {
  const [, setSearchParams] = useSearchParams();
  return useCallback(
    (rangeOption: string) =>
      setSearchParams({ [RANGE_SEARCH_PARAMETER]: rangeOption }),
    [setSearchParams],
  );
}

export function useConversationPage(): {
  conversationPage: number;
  changeConversationPage: (conversationPage: number) => void;
} {
  const [searchParams, setSearchParams] = useSearchParams();
  const conversationPage = Number(
    searchParams.get(CONVERSATION_PAGE_SEARCH_PARAMETER) ??
      FIRST_CONVERSATION_PAGE,
  );
  const changeConversationPage = useCallback(
    (nextConversationPage: number) =>
      setSearchParams((currentSearchParams) => {
        const nextSearchParams = new URLSearchParams(currentSearchParams);
        nextSearchParams.set(
          CONVERSATION_PAGE_SEARCH_PARAMETER,
          String(nextConversationPage),
        );
        return nextSearchParams;
      }),
    [setSearchParams],
  );
  return { conversationPage, changeConversationPage };
}
