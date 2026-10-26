import { ChangeEvent, useCallback, useState } from "react";
import {
  Badge,
  BannerSlim,
  Box,
  Button,
  Flex,
  Heading,
  Input,
  Select,
  Text,
} from "src/components/design-system";
import {
  EmailLogEntryDto,
  useEmailLog,
  useRetryEmail,
} from "src/api/platform/queries";
import { PlatformPagination } from "src/components/Platform/PlatformPagination";
import {
  ALL_EMAIL_STATUSES_FILTER,
  EMAIL_STATUS_BADGE_VARIANTS,
  EMAIL_STATUS_FILTERS,
  EMAIL_STATUS_NAMES,
  FIRST_PAGE,
  SEARCH_DEBOUNCE_MILLISECONDS,
  SEARCH_INPUT_STYLE,
} from "src/components/Platform/constants";
import { UNSTYLED_LIST_STYLE } from "src/components/Settings/Members/constants";
import { useDebouncedValue } from "src/utils/hooks";
import { formatTimestamp } from "src/utils/timestamp_utils";

function deliveryLabel(email: EmailLogEntryDto): string {
  const attempts =
    email.attempt_count === 1 ? "1 attempt" : `${email.attempt_count} attempts`;
  const parts = [`Created ${formatTimestamp(email.created_at)}`, attempts];
  if (email.sent_at) {
    parts.push(`sent ${formatTimestamp(email.sent_at)}`);
  }
  if (email.status === "PENDING") {
    parts.push(`next attempt ${formatTimestamp(email.next_attempt_at)}`);
  }
  return parts.join(" · ");
}

type EmailRowProps = {
  email: EmailLogEntryDto;
  onRetry: (emailId: string) => void;
  isRetrying: boolean;
};

function EmailRow({ email, onRetry, isRetrying }: EmailRowProps) {
  const handleRetry = useCallback(() => onRetry(email.id), [email.id, onRetry]);

  return (
    <Flex
      as="li"
      direction="row"
      justifyContent="space-between"
      alignItems="center"
      gap="md"
      aria-label={`${email.subject} to ${email.recipient}`}
    >
      <Flex direction="column">
        <Flex direction="row" alignItems="center" gap="xs">
          <Badge variant={EMAIL_STATUS_BADGE_VARIANTS[email.status]}>
            {EMAIL_STATUS_NAMES[email.status]}
          </Badge>
          <Text fontSize="sm">{email.subject}</Text>
        </Flex>
        <Text fontSize="xs" color="subtle">
          {`${email.recipient} · ${email.kind}`}
        </Text>
        <Text fontSize="xs" color="subtle">
          {deliveryLabel(email)}
        </Text>
        {email.last_error && (
          <Text fontSize="xs" color="error">
            {email.last_error}
          </Text>
        )}
      </Flex>
      {email.can_retry && (
        <Button
          label="Retry"
          shape="rectangle"
          size="sm"
          backgroundColor="white"
          onClick={handleRetry}
          disabled={isRetrying}
        />
      )}
    </Flex>
  );
}

export function Emails() {
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
    <Flex direction="column" gap="xl" overflow="scrollY">
      <Heading accessibilityLevel={2} size="2xl" textColor="black">
        Emails
      </Heading>
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
          <Flex as="ul" direction="column" gap="md" sx={UNSTYLED_LIST_STYLE}>
            {emails.data.emails.map((email) => (
              <EmailRow
                key={email.id}
                email={email}
                onRetry={handleRetry}
                isRetrying={retryEmail.isPending}
              />
            ))}
          </Flex>
        ))}
      {emails.data && (
        <PlatformPagination listPage={emails.data} onPageChange={setPage} />
      )}
    </Flex>
  );
}
