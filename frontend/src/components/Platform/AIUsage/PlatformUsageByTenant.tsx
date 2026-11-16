import { usePlatformUsageByTenant } from "src/api/platform_ai_usage";
import { Table, Text } from "src/components/design-system";
import {
  PLATFORM_USAGE_PAGE_SIZE,
  TENANTS_USAGE_TABLE_COLUMNS,
  TENANTS_USAGE_TABLE_HEADERS,
} from "src/components/Platform/AIUsage/constants";
import { AllowanceUsageMeter } from "src/components/Settings/AIUsage/AllowanceUsageMeter";
import { UsageListTable } from "src/components/Settings/AIUsage/UsageListTable";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { formatCount } from "src/components/Settings/AIUsage/format";
import {
  useUsageList,
  useUsageRange,
} from "src/components/Settings/AIUsage/hooks";

export function PlatformUsageByTenant() {
  const { lastDayCount } = useUsageRange();
  const { usageList, changePage, changeSort, changeSearch } = useUsageList(
    PLATFORM_USAGE_PAGE_SIZE,
  );
  const byTenant = usePlatformUsageByTenant(lastDayCount, usageList);

  if (byTenant.error) {
    return <UsageFailure failure={byTenant.error} />;
  }
  if (!byTenant.data) {
    return <UsageLoading />;
  }
  return (
    <UsageListTable
      columns={TENANTS_USAGE_TABLE_COLUMNS}
      aria-label="Platform AI usage by tenant"
      searchLabel="Search tenants"
      searchPlaceholder="Search by tenant"
      headers={TENANTS_USAGE_TABLE_HEADERS}
      usageList={usageList}
      usageListPage={byTenant.data}
      hasRows={byTenant.data.data.length > 0}
      onChangeSort={changeSort}
      onChangeSearch={changeSearch}
      onChangePage={changePage}
    >
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
            {tenant.allowance_usage && (
              <AllowanceUsageMeter allowanceUsage={tenant.allowance_usage} />
            )}
          </Table.Cell>
        </Table.Row>
      ))}
    </UsageListTable>
  );
}
