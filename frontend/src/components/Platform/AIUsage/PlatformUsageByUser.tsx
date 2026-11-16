import { usePlatformUsageByUser } from "src/api/platform_ai_usage";
import { Avatar, Table, Text } from "src/components/design-system";
import {
  PLATFORM_USAGE_PAGE_SIZE,
  USERS_USAGE_TABLE_COLUMNS,
  USERS_USAGE_TABLE_HEADERS,
} from "src/components/Platform/AIUsage/constants";
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

export function PlatformUsageByUser() {
  const { lastDayCount } = useUsageRange();
  const { usageList, changePage, changeSort, changeSearch } = useUsageList(
    PLATFORM_USAGE_PAGE_SIZE,
  );
  const byUser = usePlatformUsageByUser(lastDayCount, usageList);

  if (byUser.error) {
    return <UsageFailure failure={byUser.error} />;
  }
  if (!byUser.data) {
    return <UsageLoading />;
  }
  return (
    <UsageListTable
      columns={USERS_USAGE_TABLE_COLUMNS}
      aria-label="Platform AI usage by user"
      searchLabel="Search users"
      searchPlaceholder="Search by user or tenant"
      headers={USERS_USAGE_TABLE_HEADERS}
      usageList={usageList}
      usageListPage={byUser.data}
      hasRows={byUser.data.data.length > 0}
      onChangeSort={changeSort}
      onChangeSearch={changeSearch}
      onChangePage={changePage}
    >
      {byUser.data.data.map((user) => (
        <Table.Row
          key={`${user.tenant_id}-${user.user_id ?? user.name}`}
          aria-label={`${user.name} in ${user.tenant_name}`}
        >
          <Table.Cell>
            <Avatar name={user.name} />
            <Text fontSize="sm" truncate>
              {user.name}
            </Text>
          </Table.Cell>
          <Table.Cell>
            <Text fontSize="sm" color="subtle" truncate>
              {user.tenant_name}
            </Text>
          </Table.Cell>
          <Table.Cell align="end">
            <Text fontSize="sm">{formatCount(user.total_tokens)}</Text>
          </Table.Cell>
          <Table.Cell align="end">
            <Text fontSize="sm" color="subtle">
              {formatCount(user.call_count)}
            </Text>
          </Table.Cell>
        </Table.Row>
      ))}
    </UsageListTable>
  );
}
