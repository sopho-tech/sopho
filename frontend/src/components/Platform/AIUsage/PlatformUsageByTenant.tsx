import { useMemo } from "react";
import { usePlatformUsageByTenant } from "src/api/platform_ai_usage";
import { Flex, Table, Text } from "src/components/design-system";
import { PlatformUsagePagination } from "src/components/Platform/AIUsage/PlatformUsagePagination";
import {
  PLATFORM_USAGE_PAGE_SIZE,
  TENANTS_USAGE_TABLE_COLUMNS,
} from "src/components/Platform/AIUsage/constants";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { formatCount } from "src/components/Settings/AIUsage/format";
import {
  useConversationPage,
  useUsageRange,
} from "src/components/Settings/AIUsage/hooks";

export function PlatformUsageByTenant() {
  const { lastDayCount } = useUsageRange();
  const { conversationPage, changeConversationPage } = useConversationPage();
  const usagePage = useMemo(
    () => ({ page: conversationPage, pageSize: PLATFORM_USAGE_PAGE_SIZE }),
    [conversationPage],
  );
  const byTenant = usePlatformUsageByTenant(lastDayCount, usagePage);

  if (byTenant.error) {
    return <UsageFailure failure={byTenant.error} />;
  }
  if (!byTenant.data) {
    return <UsageLoading />;
  }
  return (
    <Flex direction="column" gap="sm">
      <Table
        columns={TENANTS_USAGE_TABLE_COLUMNS}
        aria-label="Platform AI usage by tenant"
      >
        <Table.Header>
          <Table.HeaderCell>Tenant</Table.HeaderCell>
          <Table.HeaderCell align="end">Tokens</Table.HeaderCell>
          <Table.HeaderCell align="end">Calls</Table.HeaderCell>
          <Table.HeaderCell align="end">Used of allowance</Table.HeaderCell>
        </Table.Header>
        {byTenant.data.data.map((tenant) => (
          <Table.Row key={tenant.tenant_id} aria-label={tenant.tenant_name}>
            <Table.Cell>
              <Text fontSize="sm" truncate>
                {tenant.tenant_name}
              </Text>
            </Table.Cell>
            <Table.Cell align="end">
              <Text fontSize="sm">{formatCount(tenant.total_tokens)}</Text>
            </Table.Cell>
            <Table.Cell align="end">
              <Text fontSize="sm" color="subtle">
                {formatCount(tenant.call_count)}
              </Text>
            </Table.Cell>
            <Table.Cell align="end">
              <Text fontSize="sm" color="subtle">
                {tenant.platform_ai_token_allowance !== null
                  ? `${formatCount(tenant.platform_ai_used_tokens)} of ${formatCount(tenant.platform_ai_token_allowance)}`
                  : formatCount(tenant.platform_ai_used_tokens)}
              </Text>
            </Table.Cell>
          </Table.Row>
        ))}
      </Table>
      <PlatformUsagePagination
        totalPages={byTenant.data.total_pages}
        page={byTenant.data.page}
        pageSize={byTenant.data.page_size}
        totalItems={byTenant.data.total_items}
        onChangePage={changeConversationPage}
      />
    </Flex>
  );
}
