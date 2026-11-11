import { ChangeEvent, useCallback, useState } from "react";
import {
  Badge,
  BannerSlim,
  Box,
  DropdownMenu,
  Flex,
  Heading,
  IconButton,
  Input,
  Select,
  Table,
  Text,
  ToolTip,
} from "src/components/design-system";
import {
  EmailLogEntryDto,
  useEmailLog,
  useRetryEmail,
} from "src/api/platform/queries";
import { PlatformPagination } from "src/components/Platform/PlatformPagination";
import {
  ALL_EMAIL_STATUSES_FILTER,
  EMAIL_ACTIONS_LABEL,
  EMAIL_STATUS_BADGE_VARIANTS,
  EMAIL_STATUS_FILTERS,
  EMAIL_STATUS_NAMES,
  EMAILS_TABLE_COLUMNS,
  FIRST_PAGE,
  SEARCH_DEBOUNCE_MILLISECONDS,
  SEARCH_INPUT_STYLE,
} from "src/components/Platform/constants";
import { useDebouncedValue } from "src/utils/hooks";
import { formatTimestamp } from "src/utils/timestamp_utils";

function statusDetail(email: EmailLogEntryDto): string | null {
  if (email.last_error) {
    return email.last_error;
  }
  if (email.sent_at) {
    return `Sent ${formatTimestamp(email.sent_at)}`;
  }
  if (email.status === "PENDING") {
    return `Next attempt ${formatTimestamp(email.next_attempt_at)}`;
  }
  return null;
}

function EmailStatusBadge({ email }: { email: EmailLogEntryDto }) {
  const badge = (
    <Badge variant={EMAIL_STATUS_BADGE_VARIANTS[email.status]}>
      {EMAIL_STATUS_NAMES[email.status]}
    </Badge>
  );
  const detail = statusDetail(email);
  return detail ? <ToolTip messageElement={detail}>{badge}</ToolTip> : badge;
}

type EmailRowProps = {
  email: EmailLogEntryDto;
  onRetry: (emailId: string) => void;
  isRetrying: boolean;
};

function EmailRow({ email, onRetry, isRetrying }: EmailRowProps) {
  const handleRetry = useCallback(() => onRetry(email.id), [email.id, onRetry]);

  return (
    <Table.Row aria-label={`${email.subject} to ${email.recipient}`}>
      <Table.Cell>
        <Text fontSize="sm" truncate>
          {email.recipient}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" color="darkGrey" truncate>
          {email.subject}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" color="darkGrey" truncate>
          {email.kind_display_name}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <EmailStatusBadge email={email} />
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" color="darkGrey">
          {email.attempt_count}
        </Text>
      </Table.Cell>
      <Table.Cell>
        <Text fontSize="sm" color="darkGrey">
          {formatTimestamp(email.created_at)}
        </Text>
      </Table.Cell>
      <Table.Cell align="end">
        {email.can_retry && (
          <DropdownMenu modal={false}>
            <DropdownMenu.Trigger>
              <IconButton
                type="more_horiz"
                backgroundColor="default"
                iconColor="grey"
                iconSize="md"
                disabled={isRetrying}
                aria-label={`${EMAIL_ACTIONS_LABEL} for ${email.recipient}`}
              />
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              <DropdownMenu.Item icon="refresh" onClick={handleRetry}>
                Retry sending
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu>
        )}
      </Table.Cell>
    </Table.Row>
  );
}

export function SentEmails() {
  const [statusFilter, setStatusFilter] = useState(ALL_EMAIL_STATUSES_FILTER);
  const [searchText, setSearchText] = useState("");
  const [page, setPage] = useState(FIRST_PAGE);
  const search = useDebouncedValue(searchText, SEARCH_DEBOUNCE_MILLISECONDS);
  const emails = useEmailLog({ status: statusFilter.status, search, page });
  const retryEmail = useRetryEmail();
  const { mutate: retry } = retryEmail;
  const [notice, setNotice] = useState<string | null>(null);

  const handleStatusChange = useCallback((value: string) => {
    const selectedFilter = EMAIL_STATUS_FILTERS.find(
      (filter) => filter.value === value,
    );
    if (!selectedFilter) {
      throw new Error(`Unknown email status filter ${value}`);
    }
    setStatusFilter(selectedFilter);
    setPage(FIRST_PAGE);
  }, []);
  const handleSearchChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setSearchText(event.target.value);
      setPage(FIRST_PAGE);
    },
    [],
  );
  const handleRetry = useCallback(
    (emailId: string) => {
      setNotice(null);
      retry(emailId, {
        onSuccess: () => setNotice("The email is back in the queue."),
      });
    },
    [retry],
  );

  const failure = emails.error ?? retryEmail.error;

  return (
    <Flex direction="column" gap="lg">
      <Flex direction="column" gap="2xs">
        <Heading accessibilityLevel={2} size="2xl" textColor="black">
          Sent emails
        </Heading>
        <Text fontSize="sm" color="subtle">
          Track the emails Sopho sends and retry the ones that failed.
        </Text>
      </Flex>
      <Flex direction="row" alignItems="center" gap="sm">
        <Box sx={SEARCH_INPUT_STYLE}>
          <Input
            value={searchText}
            onChange={handleSearchChange}
            placeholder="Search by recipient"
            leadingIcon="search"
            aria-label="Search by recipient"
          />
        </Box>
        <Select value={statusFilter.value} onValueChange={handleStatusChange}>
          <Select.Trigger aria-label="Status" />
          <Select.Content>
            {EMAIL_STATUS_FILTERS.map((filter) => (
              <Select.Item key={filter.value} value={filter.value}>
                {filter.label}
              </Select.Item>
            ))}
          </Select.Content>
        </Select>
      </Flex>
      {notice && <BannerSlim type="success" message={notice} />}
      {failure && <BannerSlim type="error" message={failure.message} />}
      {emails.data &&
        (emails.data.emails.length === 0 ? (
          <Text fontSize="sm" color="subtle">
            No emails match these filters.
          </Text>
        ) : (
          <Table columns={EMAILS_TABLE_COLUMNS} aria-label="Emails">
            <Table.Header>
              <Table.HeaderCell>Recipient</Table.HeaderCell>
              <Table.HeaderCell>Subject</Table.HeaderCell>
              <Table.HeaderCell>Type</Table.HeaderCell>
              <Table.HeaderCell>Status</Table.HeaderCell>
              <Table.HeaderCell>Attempts</Table.HeaderCell>
              <Table.HeaderCell>Created</Table.HeaderCell>
              <Table.HeaderCell />
            </Table.Header>
            {emails.data.emails.map((email) => (
              <EmailRow
                key={email.id}
                email={email}
                onRetry={handleRetry}
                isRetrying={retryEmail.isPending}
              />
            ))}
          </Table>
        ))}
      {emails.data && (
        <PlatformPagination listPage={emails.data} onPageChange={setPage} />
      )}
    </Flex>
  );
}
