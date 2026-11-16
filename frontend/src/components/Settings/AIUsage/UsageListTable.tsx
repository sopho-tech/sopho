import { ChangeEvent, useCallback, useEffect, useRef, useState } from "react";
import {
  UsageListParams,
  UsageSortColumn,
  UsageSortDirection,
} from "src/api/ai_usage";
import {
  Box,
  Flex,
  Input,
  Pagination,
  Table,
  Text,
} from "src/components/design-system";
import {
  NO_SEARCH_MATCH_MESSAGE,
  USAGE_SEARCH_DEBOUNCE_MILLISECONDS,
  USAGE_SEARCH_INPUT_STYLE,
} from "src/components/Settings/AIUsage/constants";
import {
  UsageListHeader,
  UsageListPage,
} from "src/components/Settings/AIUsage/datatypes";
import { useDebouncedValue } from "src/utils/hooks";

type UsageListHeaderCellProps = {
  header: UsageListHeader;
  sortColumn: UsageSortColumn;
  sortDirection: UsageSortDirection;
  onChangeSort: (sortColumn: UsageSortColumn) => void;
};

function UsageListHeaderCell({
  header,
  sortColumn,
  sortDirection,
  onChangeSort,
}: UsageListHeaderCellProps) {
  const handleSort = useCallback(
    () => onChangeSort(header.sortColumn),
    [onChangeSort, header.sortColumn],
  );
  return (
    <Table.HeaderCell
      align={header.align}
      onSort={handleSort}
      sortDirection={
        header.sortColumn === sortColumn ? sortDirection : undefined
      }
    >
      {header.label}
    </Table.HeaderCell>
  );
}

type UsageSearchInputProps = {
  search: string;
  searchLabel: string;
  searchPlaceholder: string;
  onChangeSearch: (search: string) => void;
};

function UsageSearchInput({
  search,
  searchLabel,
  searchPlaceholder,
  onChangeSearch,
}: UsageSearchInputProps) {
  const [searchText, setSearchText] = useState(search);
  const debouncedSearchText = useDebouncedValue(
    searchText,
    USAGE_SEARCH_DEBOUNCE_MILLISECONDS,
  );
  const lastSentSearchText = useRef(search);

  useEffect(() => {
    if (debouncedSearchText !== lastSentSearchText.current) {
      lastSentSearchText.current = debouncedSearchText;
      onChangeSearch(debouncedSearchText);
    }
  }, [debouncedSearchText, onChangeSearch]);

  const handleSearchChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => setSearchText(event.target.value),
    [],
  );
  return (
    <Box sx={USAGE_SEARCH_INPUT_STYLE}>
      <Input
        value={searchText}
        onChange={handleSearchChange}
        placeholder={searchPlaceholder}
        leadingIcon="search"
        aria-label={searchLabel}
      />
    </Box>
  );
}

type UsageListTableProps = {
  columns: string;
  "aria-label": string;
  searchLabel: string;
  searchPlaceholder: string;
  headers: UsageListHeader[];
  usageList: UsageListParams;
  usageListPage: UsageListPage;
  hasRows: boolean;
  onChangeSort: (sortColumn: UsageSortColumn) => void;
  onChangeSearch: (search: string) => void;
  onChangePage: (page: number) => void;
  children: React.ReactNode;
};

function ignorePageSizeChange() {}

export function UsageListTable({
  columns,
  "aria-label": ariaLabel,
  searchLabel,
  searchPlaceholder,
  headers,
  usageList,
  usageListPage,
  hasRows,
  onChangeSort,
  onChangeSearch,
  onChangePage,
  children,
}: UsageListTableProps) {
  return (
    <Flex direction="column" gap="sm">
      <UsageSearchInput
        search={usageList.search}
        searchLabel={searchLabel}
        searchPlaceholder={searchPlaceholder}
        onChangeSearch={onChangeSearch}
      />
      {hasRows ? (
        <Table columns={columns} aria-label={ariaLabel}>
          <Table.Header>
            {headers.map((header) => (
              <UsageListHeaderCell
                key={header.sortColumn}
                header={header}
                sortColumn={usageList.sortColumn}
                sortDirection={usageList.sortDirection}
                onChangeSort={onChangeSort}
              />
            ))}
          </Table.Header>
          {children}
        </Table>
      ) : (
        <Text fontSize="sm" color="subtle">
          {NO_SEARCH_MATCH_MESSAGE}
        </Text>
      )}
      <Pagination
        totalPages={usageListPage.total_pages}
        currentPage={usageListPage.page}
        pageSize={usageListPage.page_size}
        totalItems={usageListPage.total_items}
        onChangePageSize={ignorePageSizeChange}
        onPageClick={onChangePage}
        showRowsPerPage={false}
      />
    </Flex>
  );
}
