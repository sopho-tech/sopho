import { keepPreviousData, useQuery } from "@tanstack/react-query";
import {
  addUsageListSearchParams,
  UsageByKindDto,
  UsageConversationQuestionsDto,
  UsageListParams,
  UsageOverTimeDto,
  UsageSummaryDto,
} from "src/api/ai_usage";
import {
  PlatformUsageConversationDto,
  PlatformUsagePage,
  PlatformUsageTenantDto,
  PlatformUsageUserDto,
} from "src/api/platform_ai_usage/dto";
import { API_ENDPOINTS } from "src/constants/api_endpoints";
import { ApiService } from "src/utils/api_client";

export const platformAiUsageKeys = {
  all: ["platform", "ai_usage"] as const,
  summary: (lastDayCount: string) =>
    [...platformAiUsageKeys.all, "summary", lastDayCount] as const,
  overTime: (lastDayCount: string) =>
    [...platformAiUsageKeys.all, "over_time", lastDayCount] as const,
  byKind: (lastDayCount: string) =>
    [...platformAiUsageKeys.all, "by_kind", lastDayCount] as const,
  byTenant: (lastDayCount: string, usageList: UsageListParams) =>
    [...platformAiUsageKeys.all, "by_tenant", lastDayCount, usageList] as const,
  byUser: (lastDayCount: string, usageList: UsageListParams) =>
    [...platformAiUsageKeys.all, "by_user", lastDayCount, usageList] as const,
  byConversation: (lastDayCount: string, usageList: UsageListParams) =>
    [
      ...platformAiUsageKeys.all,
      "by_conversation",
      lastDayCount,
      usageList,
    ] as const,
  conversationQuestions: (
    lastDayCount: string,
    tenantId: string,
    conversationId: string,
  ) =>
    [
      ...platformAiUsageKeys.all,
      "conversation_questions",
      lastDayCount,
      tenantId,
      conversationId,
    ] as const,
};

function rangeSearchParams(lastDayCount: string): URLSearchParams {
  const searchParams = new URLSearchParams();
  searchParams.set("last_day_count", lastDayCount);
  return searchParams;
}

function pagedSearchParams(
  lastDayCount: string,
  usageList: UsageListParams,
): URLSearchParams {
  return addUsageListSearchParams(rangeSearchParams(lastDayCount), usageList);
}

async function getPlatformUsage<UsageBody>(
  endpoint: string,
  searchParams: URLSearchParams,
): Promise<UsageBody> {
  return (await ApiService.get<UsageBody>({
    url: `${endpoint}?${searchParams.toString()}`,
    onlyBody: true,
  })) as UsageBody;
}

export const usePlatformUsageSummary = (lastDayCount: string) =>
  useQuery({
    queryKey: platformAiUsageKeys.summary(lastDayCount),
    queryFn: () =>
      getPlatformUsage<UsageSummaryDto>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_SUMMARY,
        rangeSearchParams(lastDayCount),
      ),
  });

export const usePlatformUsageOverTime = (lastDayCount: string) =>
  useQuery({
    queryKey: platformAiUsageKeys.overTime(lastDayCount),
    queryFn: () =>
      getPlatformUsage<UsageOverTimeDto>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_OVER_TIME,
        rangeSearchParams(lastDayCount),
      ),
  });

export const usePlatformUsageByKind = (lastDayCount: string) =>
  useQuery({
    queryKey: platformAiUsageKeys.byKind(lastDayCount),
    queryFn: () =>
      getPlatformUsage<UsageByKindDto>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_BY_KIND,
        rangeSearchParams(lastDayCount),
      ),
  });

export const usePlatformUsageByTenant = (
  lastDayCount: string,
  usageList: UsageListParams,
) =>
  useQuery({
    queryKey: platformAiUsageKeys.byTenant(lastDayCount, usageList),
    queryFn: () =>
      getPlatformUsage<PlatformUsagePage<PlatformUsageTenantDto>>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_BY_TENANT,
        pagedSearchParams(lastDayCount, usageList),
      ),
    placeholderData: keepPreviousData,
  });

export const usePlatformUsageByUser = (
  lastDayCount: string,
  usageList: UsageListParams,
) =>
  useQuery({
    queryKey: platformAiUsageKeys.byUser(lastDayCount, usageList),
    queryFn: () =>
      getPlatformUsage<PlatformUsagePage<PlatformUsageUserDto>>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_BY_USER,
        pagedSearchParams(lastDayCount, usageList),
      ),
    placeholderData: keepPreviousData,
  });

export const usePlatformUsageByConversation = (
  lastDayCount: string,
  usageList: UsageListParams,
) =>
  useQuery({
    queryKey: platformAiUsageKeys.byConversation(lastDayCount, usageList),
    queryFn: () =>
      getPlatformUsage<PlatformUsagePage<PlatformUsageConversationDto>>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_BY_CONVERSATION,
        pagedSearchParams(lastDayCount, usageList),
      ),
    placeholderData: keepPreviousData,
  });

export const usePlatformUsageOfConversationQuestions = (
  lastDayCount: string,
  tenantId: string,
  conversationId: string,
) =>
  useQuery({
    queryKey: platformAiUsageKeys.conversationQuestions(
      lastDayCount,
      tenantId,
      conversationId,
    ),
    queryFn: () =>
      getPlatformUsage<UsageConversationQuestionsDto>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_CONVERSATION_QUESTIONS.replace(
          ":tenant_id",
          tenantId,
        ).replace(":conversation_id", conversationId),
        rangeSearchParams(lastDayCount),
      ),
  });
