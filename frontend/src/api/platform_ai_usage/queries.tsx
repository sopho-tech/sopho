import { keepPreviousData, useQuery } from "@tanstack/react-query";
import {
  UsageByKindDto,
  UsageConversationQuestionsDto,
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

type UsagePageParams = {
  page: number;
  pageSize: number;
};

export const platformAiUsageKeys = {
  all: ["platform", "ai_usage"] as const,
  summary: (lastDayCount: string) =>
    [...platformAiUsageKeys.all, "summary", lastDayCount] as const,
  overTime: (lastDayCount: string) =>
    [...platformAiUsageKeys.all, "over_time", lastDayCount] as const,
  byKind: (lastDayCount: string) =>
    [...platformAiUsageKeys.all, "by_kind", lastDayCount] as const,
  byTenant: (lastDayCount: string, usagePage: UsagePageParams) =>
    [...platformAiUsageKeys.all, "by_tenant", lastDayCount, usagePage] as const,
  byUser: (lastDayCount: string, usagePage: UsagePageParams) =>
    [...platformAiUsageKeys.all, "by_user", lastDayCount, usagePage] as const,
  byConversation: (lastDayCount: string, usagePage: UsagePageParams) =>
    [
      ...platformAiUsageKeys.all,
      "by_conversation",
      lastDayCount,
      usagePage,
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
  usagePage: UsagePageParams,
): URLSearchParams {
  const searchParams = rangeSearchParams(lastDayCount);
  searchParams.set("page", String(usagePage.page));
  searchParams.set("page_size", String(usagePage.pageSize));
  return searchParams;
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
  usagePage: UsagePageParams,
) =>
  useQuery({
    queryKey: platformAiUsageKeys.byTenant(lastDayCount, usagePage),
    queryFn: () =>
      getPlatformUsage<PlatformUsagePage<PlatformUsageTenantDto>>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_BY_TENANT,
        pagedSearchParams(lastDayCount, usagePage),
      ),
    placeholderData: keepPreviousData,
  });

export const usePlatformUsageByUser = (
  lastDayCount: string,
  usagePage: UsagePageParams,
) =>
  useQuery({
    queryKey: platformAiUsageKeys.byUser(lastDayCount, usagePage),
    queryFn: () =>
      getPlatformUsage<PlatformUsagePage<PlatformUsageUserDto>>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_BY_USER,
        pagedSearchParams(lastDayCount, usagePage),
      ),
    placeholderData: keepPreviousData,
  });

export const usePlatformUsageByConversation = (
  lastDayCount: string,
  usagePage: UsagePageParams,
) =>
  useQuery({
    queryKey: platformAiUsageKeys.byConversation(lastDayCount, usagePage),
    queryFn: () =>
      getPlatformUsage<PlatformUsagePage<PlatformUsageConversationDto>>(
        API_ENDPOINTS.PLATFORM.AI_USAGE_BY_CONVERSATION,
        pagedSearchParams(lastDayCount, usagePage),
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
