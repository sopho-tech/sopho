import { useCallback, useMemo } from "react";
import { useSearchParams } from "react-router";
import { UsageRangeParams } from "src/api/ai_usage";
import {
  ALL_KEYS_OPTION,
  CONVERSATION_PAGE_SEARCH_PARAMETER,
  DEFAULT_RANGE_OPTION,
  KEY_SEARCH_PARAMETER,
  RANGE_SEARCH_PARAMETER,
} from "src/components/Settings/AIUsage/constants";
import { FIRST_PAGE } from "src/constants/pagination";

export function useUsageRange(): UsageRangeParams {
  const [searchParams] = useSearchParams();
  const lastDayCount =
    searchParams.get(RANGE_SEARCH_PARAMETER) ?? DEFAULT_RANGE_OPTION;
  const keySource = searchParams.get(KEY_SEARCH_PARAMETER);
  return useMemo(
    () => ({ lastDayCount, keySource }),
    [lastDayCount, keySource],
  );
}

export function useChangeUsageRange(): (rangeOption: string) => void {
  const [, setSearchParams] = useSearchParams();
  return useCallback(
    (rangeOption: string) =>
      setSearchParams((currentSearchParams) => {
        const nextSearchParams = new URLSearchParams(currentSearchParams);
        nextSearchParams.set(RANGE_SEARCH_PARAMETER, rangeOption);
        nextSearchParams.delete(CONVERSATION_PAGE_SEARCH_PARAMETER);
        return nextSearchParams;
      }),
    [setSearchParams],
  );
}

export function useChangeUsageKey(): (keyOption: string) => void {
  const [, setSearchParams] = useSearchParams();
  return useCallback(
    (keyOption: string) =>
      setSearchParams((currentSearchParams) => {
        const nextSearchParams = new URLSearchParams(currentSearchParams);
        if (keyOption === ALL_KEYS_OPTION) {
          nextSearchParams.delete(KEY_SEARCH_PARAMETER);
        } else {
          nextSearchParams.set(KEY_SEARCH_PARAMETER, keyOption);
        }
        nextSearchParams.delete(CONVERSATION_PAGE_SEARCH_PARAMETER);
        return nextSearchParams;
      }),
    [setSearchParams],
  );
}

export function useConversationPage(): {
  conversationPage: number;
  changeConversationPage: (conversationPage: number) => void;
} {
  const [searchParams, setSearchParams] = useSearchParams();
  const requestedConversationPage = searchParams.get(
    CONVERSATION_PAGE_SEARCH_PARAMETER,
  );
  const conversationPage =
    requestedConversationPage === null
      ? FIRST_PAGE
      : Number(requestedConversationPage);
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
