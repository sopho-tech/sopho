import { useCallback } from "react";
import { Outlet, useLocation, useNavigate } from "react-router";
import { usePlatformUsageSummary } from "src/api/platform_ai_usage";
import { EmptyState } from "src/components/EmptyState";
import {
  BannerSlim,
  Flex,
  Heading,
  SegmentedControl,
  Select,
  Text,
} from "src/components/design-system";
import {
  PLATFORM_AI_OFF_NOTICE,
  PLATFORM_USAGE_DESCRIPTION,
  PLATFORM_USAGE_EMPTY_DESCRIPTION,
  PLATFORM_USAGE_EMPTY_HEADING,
  PLATFORM_USAGE_TABS,
} from "src/components/Platform/AIUsage/constants";
import { RANGE_OPTIONS } from "src/components/Settings/AIUsage/constants";
import { formatUtcDay } from "src/components/Settings/AIUsage/format";
import {
  useChangeUsageRange,
  searchWithoutUsageList,
  useUsageRange,
} from "src/components/Settings/AIUsage/hooks";

export function PlatformAIUsage() {
  const navigate = useNavigate();
  const location = useLocation();
  const { lastDayCount } = useUsageRange();
  const changeUsageRange = useChangeUsageRange();
  const summary = usePlatformUsageSummary(lastDayCount);

  const chosenTab = PLATFORM_USAGE_TABS.find((usageTab) =>
    location.pathname.startsWith(usageTab.value),
  );
  const handleChooseTab = useCallback(
    (usageTabPath: string) =>
      void navigate({
        pathname: usageTabPath,
        search: searchWithoutUsageList(location.search),
      }),
    [navigate, location.search],
  );

  return (
    <Flex direction="column" gap="lg">
      <Flex
        direction="row"
        justifyContent="space-between"
        alignItems="flex-start"
        gap="md"
      >
        <Flex direction="column" gap="2xs">
          <Heading accessibilityLevel={2} size="2xl" textColor="black">
            Platform AI Usage
          </Heading>
          <Text fontSize="sm" color="subtle">
            {PLATFORM_USAGE_DESCRIPTION}
          </Text>
        </Flex>
        <Flex direction="column" alignItems="flex-end" gap="2xs">
          <Select value={lastDayCount} onValueChange={changeUsageRange}>
            <Select.Trigger aria-label="Date range" />
            <Select.Content>
              {RANGE_OPTIONS.map((rangeOption) => (
                <Select.Item key={rangeOption.value} value={rangeOption.value}>
                  {rangeOption.label}
                </Select.Item>
              ))}
            </Select.Content>
          </Select>
          {summary.data && (
            <Text fontSize="xs" color="subtle">
              {`${formatUtcDay(summary.data.from)} to ${formatUtcDay(summary.data.to)}`}
            </Text>
          )}
        </Flex>
      </Flex>
      {summary.data && !summary.data.is_platform_ai_on && (
        <BannerSlim type="info" message={PLATFORM_AI_OFF_NOTICE} />
      )}
      <Flex direction="row">
        <SegmentedControl
          options={PLATFORM_USAGE_TABS}
          value={chosenTab ? chosenTab.value : ""}
          onValueChange={handleChooseTab}
          size="md"
        />
      </Flex>
      {summary.data && !summary.data.has_usage ? (
        <EmptyState
          icon="bar_chart"
          heading={PLATFORM_USAGE_EMPTY_HEADING}
          description={PLATFORM_USAGE_EMPTY_DESCRIPTION}
        />
      ) : (
        <Outlet />
      )}
    </Flex>
  );
}
