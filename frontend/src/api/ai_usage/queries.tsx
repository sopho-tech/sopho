import { keepPreviousData, useQuery } from "@tanstack/react-query";
import {
  UsageByConversationDto,
  UsageByKindDto,
  UsageByUserDto,
  UsageConversationQuestionsDto,
  UsageListParams,
  UsageOverTimeDto,
  UsageRangeParams,
  UsageSummaryDto,
} from "src/api/ai_usage/dto";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { ApiService } from "src/utils/api_client";
import { tenantApiUrl, useTenantId } from "src/utils/tenant";

export const aiUsageKeys = {
  all: (tenantId: string) => [tenantId, "ai_usage"] as const,
  summary: (tenantId: string, range: UsageRangeParams) =>
    [...aiUsageKeys.all(tenantId), "summary", range] as const,
  overTime: (tenantId: string, range: UsageRangeParams) =>
    [...aiUsageKeys.all(tenantId), "over_time", range] as const,
  byUser: (
    tenantId: string,
    range: UsageRangeParams,
    usageList: UsageListParams,
  ) => [...aiUsageKeys.all(tenantId), "by_user", range, usageList] as const,
  byKind: (tenantId: string, range: UsageRangeParams) =>
    [...aiUsageKeys.all(tenantId), "by_kind", range] as const,
  byConversation: (
    tenantId: string,
    range: UsageRangeParams,
    usageList: UsageListParams,
  ) =>
    [
      ...aiUsageKeys.all(tenantId),
      "by_conversation",
      range,
      usageList,
    ] as const,
  conversationQuestions: (
    tenantId: string,
    range: UsageRangeParams,
    conversationId: string,
  ) =>
    [
      ...aiUsageKeys.all(tenantId),
      "conversation_questions",
      range,
      conversationId,
    ] as const,
};

function rangeSearchParams(range: UsageRangeParams): URLSearchParams {
  const searchParams = new URLSearchParams();
  searchParams.set("last_day_count", range.lastDayCount);
  if (range.keySource !== null) {
    searchParams.set("key_source", range.keySource);
  }
  return searchParams;
}

export function addUsageListSearchParams(
  searchParams: URLSearchParams,
  usageList: UsageListParams,
): URLSearchParams {
  searchParams.set("page", String(usageList.page));
  searchParams.set("page_size", String(usageList.pageSize));
  searchParams.set("sort_by", usageList.sortColumn);
  searchParams.set("sort_direction", usageList.sortDirection);
  if (usageList.search !== "") {
    searchParams.set("search", usageList.search);
  }
  return searchParams;
}

function usageUrl(
  tenantId: string,
  endpoint: string,
  searchParams: URLSearchParams,
): string {
  return `${tenantApiUrl(tenantId, endpoint)}?${searchParams.toString()}`;
}

async function getUsage<UsageBody>(url: string): Promise<UsageBody> {
  return (await ApiService.get<UsageBody>({
    url,
    onlyBody: true,
  })) as UsageBody;
}

export const useUsageSummary = (range: UsageRangeParams) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: aiUsageKeys.summary(tenantId, range),
    queryFn: () =>
      getUsage<UsageSummaryDto>(
        usageUrl(
          tenantId,
          API_ENDPOINTS.AI_USAGE.SUMMARY,
          rangeSearchParams(range),
        ),
      ),
  });
};

export const useUsageOverTime = (range: UsageRangeParams) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: aiUsageKeys.overTime(tenantId, range),
    queryFn: () =>
      getUsage<UsageOverTimeDto>(
        usageUrl(
          tenantId,
          API_ENDPOINTS.AI_USAGE.OVER_TIME,
          rangeSearchParams(range),
        ),
      ),
  });
};

export const useUsageByUser = (
  range: UsageRangeParams,
  usageList: UsageListParams,
) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: aiUsageKeys.byUser(tenantId, range, usageList),
    queryFn: () =>
      getUsage<UsageByUserDto>(
        usageUrl(
          tenantId,
          API_ENDPOINTS.AI_USAGE.BY_USER,
          addUsageListSearchParams(rangeSearchParams(range), usageList),
        ),
      ),
    placeholderData: keepPreviousData,
  });
};

export const useUsageByKind = (range: UsageRangeParams) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: aiUsageKeys.byKind(tenantId, range),
    queryFn: () =>
      getUsage<UsageByKindDto>(
        usageUrl(
          tenantId,
          API_ENDPOINTS.AI_USAGE.BY_KIND,
          rangeSearchParams(range),
        ),
      ),
  });
};

export const useUsageByConversation = (
  range: UsageRangeParams,
  usageList: UsageListParams,
) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: aiUsageKeys.byConversation(tenantId, range, usageList),
    queryFn: () =>
      getUsage<UsageByConversationDto>(
        usageUrl(
          tenantId,
          API_ENDPOINTS.AI_USAGE.BY_CONVERSATION,
          addUsageListSearchParams(rangeSearchParams(range), usageList),
        ),
      ),
    placeholderData: keepPreviousData,
  });
};

export const useUsageOfConversationQuestions = (
  range: UsageRangeParams,
  conversationId: string,
) => {
  const tenantId = useTenantId();
  return useQuery({
    queryKey: aiUsageKeys.conversationQuestions(
      tenantId,
      range,
      conversationId,
    ),
    queryFn: () =>
      getUsage<UsageConversationQuestionsDto>(
        usageUrl(
          tenantId,
          API_ENDPOINTS.AI_USAGE.CONVERSATION_QUESTIONS.replace(
            ":conversation_id",
            conversationId,
          ),
          rangeSearchParams(range),
        ),
      ),
  });
};
