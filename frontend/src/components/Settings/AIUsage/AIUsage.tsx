import { useCallback } from "react";
import { Outlet, useLocation, useNavigate } from "react-router";
import { useUsageSummary } from "src/api/ai_usage";
import { EmptyState } from "src/components/EmptyState";
import {
  Flex,
  Heading,
  SegmentedControl,
  Select,
  Text,
} from "src/components/design-system";
import {
  ALL_KEYS_OPTION,
  EMPTY_DESCRIPTION,
  EMPTY_HEADING,
  KEY_OPTIONS,
  PAGE_DESCRIPTION,
  RANGE_OPTIONS,
  USAGE_TABS,
} from "src/components/Settings/AIUsage/constants";
import { formatUtcDay } from "src/components/Settings/AIUsage/format";
import {
  useChangeUsageKey,
  useChangeUsageRange,
  searchWithoutUsageList,
  useUsageRange,
} from "src/components/Settings/AIUsage/hooks";
import { useTenantPath } from "src/utils/tenant";

export function AIUsage() {
  const navigate = useNavigate();
  const location = useLocation();
  const toTenantPath = useTenantPath();
  const range = useUsageRange();
  const changeUsageRange = useChangeUsageRange();
  const changeUsageKey = useChangeUsageKey();
  const summary = useUsageSummary(range);

  const chosenTab = USAGE_TABS.find((usageTab) =>
    location.pathname.startsWith(toTenantPath(usageTab.value)),
  );
  const handleChooseTab = useCallback(
    (usageTabPath: string) =>
      void navigate({
        pathname: toTenantPath(usageTabPath),
        search: searchWithoutUsageList(location.search),
      }),
    [navigate, toTenantPath, location.search],
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
            AI Usage
          </Heading>
          <Text fontSize="sm" color="subtle">
            {PAGE_DESCRIPTION}
          </Text>
        </Flex>
        <Flex direction="column" alignItems="flex-end" gap="2xs">
          <Flex direction="row" alignItems="center" gap="sm">
            {(summary.data?.is_platform_ai_on || range.keySource !== null) && (
              <Select
                value={range.keySource ?? ALL_KEYS_OPTION}
                onValueChange={changeUsageKey}
              >
                <Select.Trigger aria-label="AI key" />
                <Select.Content>
                  {KEY_OPTIONS.map((keyOption) => (
                    <Select.Item key={keyOption.value} value={keyOption.value}>
                      {keyOption.label}
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select>
            )}
            <Select value={range.lastDayCount} onValueChange={changeUsageRange}>
              <Select.Trigger aria-label="Date range" />
              <Select.Content>
                {RANGE_OPTIONS.map((rangeOption) => (
                  <Select.Item
                    key={rangeOption.value}
                    value={rangeOption.value}
                  >
                    {rangeOption.label}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select>
          </Flex>
          {summary.data && (
            <Text fontSize="xs" color="subtle">
              {`${formatUtcDay(summary.data.from)} to ${formatUtcDay(summary.data.to)}`}
            </Text>
          )}
        </Flex>
      </Flex>
      <Flex direction="row">
        <SegmentedControl
          options={USAGE_TABS}
          value={chosenTab ? chosenTab.value : ""}
          onValueChange={handleChooseTab}
          size="md"
        />
      </Flex>
      {summary.data && !summary.data.has_usage ? (
        <EmptyState
          icon="bar_chart"
          heading={EMPTY_HEADING}
          description={EMPTY_DESCRIPTION}
        />
      ) : (
        <Outlet />
      )}
    </Flex>
  );
}
