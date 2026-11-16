import { useCallback } from "react";
import { useLocation, useNavigate } from "react-router";
import { UsageConversationDto, useUsageByConversation } from "src/api/ai_usage";
import { Avatar, Table, Text } from "src/components/design-system";
import { UsageListTable } from "src/components/Settings/AIUsage/UsageListTable";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import {
  CONVERSATION_ID_PATH_PARAMETER,
  CONVERSATION_PAGE_SIZE,
  CONVERSATIONS_TABLE_COLUMNS,
  CONVERSATIONS_TABLE_HEADERS,
} from "src/components/Settings/AIUsage/constants";
import { formatCount } from "src/components/Settings/AIUsage/format";
import {
  useUsageList,
  useUsageRange,
} from "src/components/Settings/AIUsage/hooks";
import { APP_ROUTES } from "src/constants/app_routes";
import { useTenantPath } from "src/utils/tenant";

type ConversationRowProps = {
  conversation: UsageConversationDto;
  onOpen: (conversation: UsageConversationDto) => void;
};

function ConversationRow({ conversation, onOpen }: ConversationRowProps) {
  const handleOpen = useCallback(
    () => onOpen(conversation),
    [conversation, onOpen],
  );
  return (
    <Table.Row aria-label={conversation.name} onClick={handleOpen}>
      <Table.Cell>
        <Text fontSize="sm" truncate>
          {conversation.name}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Avatar name={conversation.user_name} />
        <Text fontSize="sm" truncate>
          {conversation.user_name}
        </Text>
      </Table.Cell>
      <Table.Cell align="end">
        <Text fontSize="sm">{formatCount(conversation.total_tokens)}</Text>
      </Table.Cell>
      <Table.Cell align="end">
        <Text fontSize="sm" color="subtle">
          {formatCount(conversation.call_count)}
        </Text>
      </Table.Cell>
    </Table.Row>
  );
}

export function UsageByConversation() {
  const navigate = useNavigate();
  const location = useLocation();
  const toTenantPath = useTenantPath();
  const range = useUsageRange();
  const { usageList, changePage, changeSort, changeSearch } = useUsageList(
    CONVERSATION_PAGE_SIZE,
  );
  const byConversation = useUsageByConversation(range, usageList);

  const handleOpenConversation = useCallback(
    (conversation: UsageConversationDto) =>
      void navigate({
        pathname: toTenantPath(
          APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_CONVERSATION.replace(
            CONVERSATION_ID_PATH_PARAMETER,
            conversation.conversation_id,
          ),
        ),
        search: location.search,
      }),
    [navigate, toTenantPath, location.search],
  );

  if (byConversation.error) {
    return <UsageFailure failure={byConversation.error} />;
  }
  if (!byConversation.data) {
    return <UsageLoading />;
  }
  return (
    <UsageListTable
      columns={CONVERSATIONS_TABLE_COLUMNS}
      aria-label="Usage by conversation"
      searchLabel="Search conversations"
      searchPlaceholder="Search by conversation or user"
      headers={CONVERSATIONS_TABLE_HEADERS}
      usageList={usageList}
      usageListPage={byConversation.data}
      hasRows={byConversation.data.data.length > 0}
      onChangeSort={changeSort}
      onChangeSearch={changeSearch}
      onChangePage={changePage}
    >
      {byConversation.data.data.map((conversation) => (
        <ConversationRow
          key={conversation.conversation_id}
          conversation={conversation}
          onOpen={handleOpenConversation}
        />
      ))}
    </UsageListTable>
  );
}
