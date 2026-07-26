import { useCallback, useEffect, useMemo, useState } from "react";
import { generatePath, useNavigate } from "react-router";
import { PaginationState } from "@tanstack/react-table";
import { Flex, Spinner, Text } from "src/components/design-system";
import { SophoTable, TableType } from "src/components/SophoTable";
import {
  ConversationListItemDto,
  useBulkDeleteConversations,
  useConversations,
} from "src/api/conversational_analytics";
import { APP_ROUTES } from "src/constants/app_routes";
import {
  CONVERSATION_LIST_PAGE_SIZE,
  CONVERSATION_SEARCH_DEBOUNCE_MS,
} from "src/components/ConversationalAnalytics/constants";
import { useDebouncedValue } from "src/utils/hooks";
import { ConversationListHeader } from "./ConversationListHeader";
import { BulkDeleteConversationsDialog } from "./BulkDeleteConversationsDialog";
import { createConversationListColumns } from "./conversationListColumns";
import { useConversationSelection } from "./useConversationSelection";

export const ConversationList = () => {
  const navigate = useNavigate();
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: CONVERSATION_LIST_PAGE_SIZE,
  });
  const [search, setSearch] = useState("");
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const debouncedSearch = useDebouncedValue(
    search,
    CONVERSATION_SEARCH_DEBOUNCE_MS,
  );
  const selection = useConversationSelection();
  const bulkDelete = useBulkDeleteConversations();

  const { data, isLoading, isError } = useConversations({
    page: pagination.pageIndex,
    pageSize: pagination.pageSize,
    search: debouncedSearch,
  });

  const conversations = useMemo(() => data?.items ?? [], [data?.items]);
  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / pagination.pageSize));
  const clearSelection = selection.clear;

  useEffect(() => {
    setPagination((previous) => ({ ...previous, pageIndex: 0 }));
  }, [debouncedSearch]);

  useEffect(() => {
    clearSelection();
  }, [pagination.pageIndex, debouncedSearch, clearSelection]);

  const { isSelectMode, toggle } = selection;

  const handleRowClick = useCallback(
    (conversation: ConversationListItemDto) => {
      if (isSelectMode) {
        toggle(conversation.id);
        return;
      }
      navigate(
        generatePath(APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION, {
          id: conversation.id,
        }),
      );
    },
    [navigate, isSelectMode, toggle],
  );

  const columns = useMemo(
    () =>
      createConversationListColumns({
        isSelectMode: selection.isSelectMode,
        selectedIds: selection.selectedIds,
        onEnterSelectMode: selection.enterSelectMode,
      }),
    [selection.isSelectMode, selection.selectedIds, selection.enterSelectMode],
  );

  const paginationConfig = useMemo(
    () => ({
      pagination,
      totalItems: total,
      totalPages,
      currentPage: pagination.pageIndex,
      pageSize: pagination.pageSize,
      onPaginationChange: setPagination,
      onChangePageSize: (newValue: string) =>
        setPagination({ pageIndex: 0, pageSize: Number(newValue) }),
      onPageClick: (newPage: number) =>
        setPagination((previous) => ({ ...previous, pageIndex: newPage })),
    }),
    [pagination, total, totalPages],
  );

  const isRowSelected = useCallback(
    (conversation: ConversationListItemDto) =>
      selection.selectedIds.has(conversation.id),
    [selection.selectedIds],
  );

  const handleConfirmDelete = () => {
    bulkDelete.mutate([...selection.selectedIds], {
      onSuccess: () => {
        setIsDeleteDialogOpen(false);
        selection.exitSelectMode();
      },
    });
  };

  const isEmpty = !isLoading && !isError && conversations.length === 0;

  return (
    <Flex direction="column" flex="grow" overflow="hidden">
      <ConversationListHeader
        search={search}
        onSearchChange={setSearch}
        isSelectMode={selection.isSelectMode}
        selectedCount={selection.selectedIds.size}
        onEnterSelectMode={() => selection.enterSelectMode()}
        onExitSelectMode={selection.exitSelectMode}
        onSelectAll={() =>
          selection.selectAll(conversations.map((item) => item.id))
        }
        onDeleteSelected={() => setIsDeleteDialogOpen(true)}
        onCreateConversation={() =>
          navigate(APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.INDEX)
        }
      />

      {isEmpty ? (
        <Flex justifyContent="center" paddingY="xl">
          <Text color="subtle" fontSize="sm">
            {debouncedSearch.trim()
              ? "No conversations match your search."
              : "No conversations yet."}
          </Text>
        </Flex>
      ) : (
        <SophoTable
          tableType={TableType.PAGINATED}
          variant="plain"
          showHeader={false}
          columns={columns}
          data={conversations}
          isLoading={isLoading}
          isError={isError}
          loadingComponent={
            <Flex justifyContent="center" paddingY="xl">
              <Spinner size="md" color="grey" />
            </Flex>
          }
          onRowClick={handleRowClick}
          isRowSelected={isRowSelected}
          paginationConfig={paginationConfig}
        />
      )}

      <BulkDeleteConversationsDialog
        count={selection.selectedIds.size}
        open={isDeleteDialogOpen}
        onOpenChange={setIsDeleteDialogOpen}
        onClose={() => setIsDeleteDialogOpen(false)}
        onConfirm={handleConfirmDelete}
        isPending={bulkDelete.isPending}
      />
    </Flex>
  );
};
