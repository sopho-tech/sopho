import { useUsageByUser } from "src/api/ai_usage";
import { Avatar, Flex, Table, Text } from "src/components/design-system";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { USERS_TABLE_COLUMNS } from "src/components/Settings/AIUsage/constants";
import { formatCount } from "src/components/Settings/AIUsage/format";
import { useUsageRange } from "src/components/Settings/AIUsage/hooks";

export function UsageByUser() {
  const byUser = useUsageByUser(useUsageRange());

  if (byUser.error) {
    return <UsageFailure failure={byUser.error} />;
  }
  if (!byUser.data) {
    return <UsageLoading />;
  }
  return (
    <Flex direction="column" gap="lg">
      <Table columns={USERS_TABLE_COLUMNS} aria-label="Usage by user">
        <Table.Header>
          <Table.HeaderCell>User</Table.HeaderCell>
          <Table.HeaderCell align="end">Tokens</Table.HeaderCell>
          <Table.HeaderCell align="end">Calls</Table.HeaderCell>
        </Table.Header>
        {byUser.data.users.map((user) => (
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
      </Table>
    </Flex>
  );
}
