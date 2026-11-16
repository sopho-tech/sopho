import { useMemo } from "react";
import { usePlatformUsageByUser } from "src/api/platform_ai_usage";
import { Avatar, Flex, Table, Text } from "src/components/design-system";
import { PlatformUsagePagination } from "src/components/Platform/AIUsage/PlatformUsagePagination";
import {
  PLATFORM_USAGE_PAGE_SIZE,
  USERS_USAGE_TABLE_COLUMNS,
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

export function PlatformUsageByUser() {
  const { lastDayCount } = useUsageRange();
  const { conversationPage, changeConversationPage } = useConversationPage();
  const usagePage = useMemo(
    () => ({ page: conversationPage, pageSize: PLATFORM_USAGE_PAGE_SIZE }),
    [conversationPage],
  );
  const byUser = usePlatformUsageByUser(lastDayCount, usagePage);

  if (byUser.error) {
    return <UsageFailure failure={byUser.error} />;
  }
  if (!byUser.data) {
    return <UsageLoading />;
  }
  return (
    <Flex direction="column" gap="sm">
      <Table
        columns={USERS_USAGE_TABLE_COLUMNS}
        aria-label="Platform AI usage by user"
      >
        <Table.Header>
          <Table.HeaderCell>User</Table.HeaderCell>
          <Table.HeaderCell>Tenant</Table.HeaderCell>
          <Table.HeaderCell align="end">Tokens</Table.HeaderCell>
          <Table.HeaderCell align="end">Calls</Table.HeaderCell>
        </Table.Header>
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
      </Table>
      <PlatformUsagePagination
        totalPages={byUser.data.total_pages}
        page={byUser.data.page}
        pageSize={byUser.data.page_size}
        totalItems={byUser.data.total_items}
        onChangePage={changeConversationPage}
      />
    </Flex>
  );
}
