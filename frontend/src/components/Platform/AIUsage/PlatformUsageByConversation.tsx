import { useCallback } from "react";
import { useLocation, useNavigate } from "react-router";
import {
  PlatformUsageConversationDto,
  usePlatformUsageByConversation,
} from "src/api/platform_ai_usage";
import { Avatar, Table, Text } from "src/components/design-system";
import {
  CONVERSATION_ID_PATH_PARAMETER,
  CONVERSATIONS_USAGE_TABLE_COLUMNS,
  CONVERSATIONS_USAGE_TABLE_HEADERS,
  PLATFORM_USAGE_PAGE_SIZE,
  TENANT_ID_PATH_PARAMETER,
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
import { APP_ROUTES } from "src/constants/app_routes";

type ConversationRowProps = {
  conversation: PlatformUsageConversationDto;
  onOpen: (conversation: PlatformUsageConversationDto) => void;
};

function ConversationRow({ conversation, onOpen }: ConversationRowProps) {
  const handleOpen = useCallback(
    () => onOpen(conversation),
    [conversation, onOpen],
  );
  return (
    <Table.Row
      aria-label={`${conversation.name} in ${conversation.tenant_name}`}
      onClick={handleOpen}
    >
      <Table.Cell>
        <Text fontSize="sm" truncate>
          {conversation.name}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" color="subtle" truncate>
          {conversation.tenant_name}
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

export function PlatformUsageByConversation() {
  const navigate = useNavigate();
  const location = useLocation();
  const { lastDayCount } = useUsageRange();
  const { usageList, changePage, changeSort, changeSearch } = useUsageList(
    PLATFORM_USAGE_PAGE_SIZE,
  );
  const byConversation = usePlatformUsageByConversation(
    lastDayCount,
    usageList,
  );

  const handleOpenConversation = useCallback(
    (conversation: PlatformUsageConversationDto) =>
      void navigate({
        pathname: APP_ROUTES.PLATFORM_ROUTES.AI_USAGE_CONVERSATION.replace(
          TENANT_ID_PATH_PARAMETER,
          conversation.tenant_id,
        ).replace(CONVERSATION_ID_PATH_PARAMETER, conversation.conversation_id),
        search: location.search,
      }),
    [navigate, location.search],
  );

  if (byConversation.error) {
    return <UsageFailure failure={byConversation.error} />;
  }
  if (!byConversation.data) {
    return <UsageLoading />;
  }
  return (
    <UsageListTable
      columns={CONVERSATIONS_USAGE_TABLE_COLUMNS}
      aria-label="Platform AI usage by conversation"
      searchLabel="Search conversations"
      searchPlaceholder="Search by conversation, tenant or user"
      headers={CONVERSATIONS_USAGE_TABLE_HEADERS}
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
