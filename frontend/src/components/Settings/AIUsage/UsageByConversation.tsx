import { useCallback, useMemo } from "react";
import { useLocation, useNavigate } from "react-router";
import { UsageConversationDto, useUsageByConversation } from "src/api/ai_usage";
import {
  Avatar,
  Flex,
  Pagination,
  Table,
  Text,
} from "src/components/design-system";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import {
  CONVERSATION_ID_PATH_PARAMETER,
  CONVERSATION_PAGE_SIZE,
  CONVERSATIONS_TABLE_COLUMNS,
} from "src/components/Settings/AIUsage/constants";
import { formatCount } from "src/components/Settings/AIUsage/format";
import {
  useConversationPage,
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

function ignorePageSizeChange() {}

export function UsageByConversation() {
  const navigate = useNavigate();
  const location = useLocation();
  const toTenantPath = useTenantPath();
  const range = useUsageRange();
  const { conversationPage, changeConversationPage } = useConversationPage();
  const conversationPageParams = useMemo(
    () => ({ page: conversationPage, pageSize: CONVERSATION_PAGE_SIZE }),
    [conversationPage],
  );
  const byConversation = useUsageByConversation(range, conversationPageParams);

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
    <Flex direction="column" gap="lg">
      <Flex direction="column" gap="sm">
        <Table
          columns={CONVERSATIONS_TABLE_COLUMNS}
          aria-label="Usage by conversation"
        >
          <Table.Header>
            <Table.HeaderCell>Conversation</Table.HeaderCell>
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
        <Pagination
          totalPages={byConversation.data.total_pages}
          currentPage={byConversation.data.page}
          pageSize={byConversation.data.page_size}
          totalItems={byConversation.data.total_items}
          onChangePageSize={ignorePageSizeChange}
          onPageClick={changeConversationPage}
          showRowsPerPage={false}
        />
      </Flex>
    </Flex>
  );
}
