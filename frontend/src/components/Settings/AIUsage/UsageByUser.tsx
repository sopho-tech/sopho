import { useUsageByUser } from "src/api/ai_usage";
import { Avatar, Table, Text } from "src/components/design-system";
import { UsageListTable } from "src/components/Settings/AIUsage/UsageListTable";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import {
  CONVERSATION_PAGE_SIZE,
  USERS_TABLE_COLUMNS,
  USERS_TABLE_HEADERS,
} from "src/components/Settings/AIUsage/constants";
import { formatCount } from "src/components/Settings/AIUsage/format";
import {
  useUsageList,
  useUsageRange,
} from "src/components/Settings/AIUsage/hooks";

export function UsageByUser() {
  const { usageList, changePage, changeSort, changeSearch } = useUsageList(
    CONVERSATION_PAGE_SIZE,
  );
  const byUser = useUsageByUser(useUsageRange(), usageList);

  if (byUser.error) {
    return <UsageFailure failure={byUser.error} />;
  }
  if (!byUser.data) {
    return <UsageLoading />;
  }
  return (
    <UsageListTable
      columns={USERS_TABLE_COLUMNS}
      aria-label="Usage by user"
      searchLabel="Search users"
      searchPlaceholder="Search by user"
      headers={USERS_TABLE_HEADERS}
      usageList={usageList}
      usageListPage={byUser.data}
      hasRows={byUser.data.data.length > 0}
      onChangeSort={changeSort}
      onChangeSearch={changeSearch}
      onChangePage={changePage}
    >
      {byUser.data.data.map((user) => (
        <Table.Row key={user.user_id ?? user.name} aria-label={user.name}>
          <Table.Cell>
            <Avatar name={user.name} />
            <Text fontSize="sm" truncate>
              {user.name}
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
