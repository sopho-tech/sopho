import { useCallback, useMemo } from "react";
import { useLocation, useNavigate } from "react-router";
import {
  PlatformUsageConversationDto,
  usePlatformUsageByConversation,
} from "src/api/platform_ai_usage";
import { Avatar, Flex, Table, Text } from "src/components/design-system";
import { PlatformUsagePagination } from "src/components/Platform/AIUsage/PlatformUsagePagination";
import {
  CONVERSATION_ID_PATH_PARAMETER,
  CONVERSATIONS_USAGE_TABLE_COLUMNS,
  PLATFORM_USAGE_PAGE_SIZE,
  TENANT_ID_PATH_PARAMETER,
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
  const { conversationPage, changeConversationPage } = useConversationPage();
  const usagePage = useMemo(
    () => ({ page: conversationPage, pageSize: PLATFORM_USAGE_PAGE_SIZE }),
    [conversationPage],
  );
  const byConversation = usePlatformUsageByConversation(
    lastDayCount,
    usagePage,
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
    <Flex direction="column" gap="sm">
      <Table
        columns={CONVERSATIONS_USAGE_TABLE_COLUMNS}
        aria-label="Platform AI usage by conversation"
      >
        <Table.Header>
          <Table.HeaderCell>Conversation</Table.HeaderCell>
          <Table.HeaderCell>Tenant</Table.HeaderCell>
          <Table.HeaderCell>User</Table.HeaderCell>
          <Table.HeaderCell align="end">Tokens</Table.HeaderCell>
          <Table.HeaderCell align="end">Calls</Table.HeaderCell>
        </Table.Header>
        {byConversation.data.data.map((conversation) => (
          <ConversationRow
            key={conversation.conversation_id}
            conversation={conversation}
            onOpen={handleOpenConversation}
          />
        ))}
      </Table>
      <PlatformUsagePagination
        totalPages={byConversation.data.total_pages}
        page={byConversation.data.page}
        pageSize={byConversation.data.page_size}
        totalItems={byConversation.data.total_items}
        onChangePage={changeConversationPage}
      />
    </Flex>
  );
}
