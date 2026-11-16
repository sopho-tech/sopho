import { useCallback } from "react";
import { useLocation, useNavigate, useParams } from "react-router";
import {
  UsageConversationQuestionsDto,
  useUsageOfConversationQuestions,
} from "src/api/ai_usage";
import {
  BackButton,
  Flex,
  Grid,
  GridItem,
  Heading,
  Table,
  Text,
} from "src/components/design-system";
import {
  UsageFailure,
  UsageLoading,
} from "src/components/Settings/AIUsage/UsageStates";
import { UsageTile } from "src/components/Settings/AIUsage/UsageTiles";
import {
  GRID_GUTTER,
  QUESTION_TILE_COLUMN_SPAN,
  QUESTIONS_TABLE_COLUMNS,
} from "src/components/Settings/AIUsage/constants";
import {
  formatCount,
  formatUtcMoment,
} from "src/components/Settings/AIUsage/format";
import { useUsageRange } from "src/components/Settings/AIUsage/hooks";
import { APP_ROUTES } from "src/constants/app_routes";
import { useTenantPath } from "src/utils/tenant";

type ConversationQuestionsProps = {
  conversationQuestions: UsageConversationQuestionsDto;
  onBack: () => void;
};

export function ConversationQuestions({
  conversationQuestions,
  onBack,
}: ConversationQuestionsProps) {
  return (
    <Flex direction="column" gap="lg">
      <Flex direction="row" alignItems="center" gap="xs">
        <BackButton label="All conversations" onClick={onBack} />
        <Heading accessibilityLevel={3} size="xl" textColor="black">
          {conversationQuestions.name}
        </Heading>
      </Flex>
      <Grid as="div" columnGutter={GRID_GUTTER} rowGutter={GRID_GUTTER}>
        <GridItem as="div" colSpan={QUESTION_TILE_COLUMN_SPAN}>
          <UsageTile
            label="Tokens"
            value={formatCount(conversationQuestions.total_tokens)}
            detail="In this conversation"
          />
        </GridItem>
        <GridItem as="div" colSpan={QUESTION_TILE_COLUMN_SPAN}>
          <UsageTile
            label="Questions"
            value={formatCount(conversationQuestions.question_count)}
            detail="Asked in the range"
          />
        </GridItem>
        <GridItem as="div" colSpan={QUESTION_TILE_COLUMN_SPAN}>
          <UsageTile
            label="Model calls"
            value={formatCount(conversationQuestions.call_count)}
            detail="Requests to the AI provider"
          />
        </GridItem>
      </Grid>
      <Table
        columns={QUESTIONS_TABLE_COLUMNS}
        aria-label={`Questions of ${conversationQuestions.name}`}
      >
        <Table.Header>
          <Table.HeaderCell>Question</Table.HeaderCell>
          <Table.HeaderCell align="end">Tokens</Table.HeaderCell>
          <Table.HeaderCell align="end">Calls</Table.HeaderCell>
        </Table.Header>
        {conversationQuestions.questions.map((question) => (
          <Table.Row
            key={question.conversation_message_id ?? question.text_kind}
            aria-label={question.text}
          >
            <Table.Cell>
              <Flex direction="column" overflow="hidden">
                <Text fontSize="sm">{question.text}</Text>
                <Text fontSize="xs" color="subtle">
                  {formatUtcMoment(question.first_call_at)}
                </Text>
              </Flex>
            </Table.Cell>
            <Table.Cell align="end">
              <Text fontSize="sm">{formatCount(question.total_tokens)}</Text>
            </Table.Cell>
            <Table.Cell align="end">
              <Text fontSize="sm" color="subtle">
                {formatCount(question.call_count)}
              </Text>
            </Table.Cell>
          </Table.Row>
        ))}
      </Table>
    </Flex>
  );
}

export function UsageConversationQuestions() {
  const { conversationId } = useParams();
  if (!conversationId) {
    throw new Error("The conversation id is missing from the address");
  }
  const navigate = useNavigate();
  const location = useLocation();
  const toTenantPath = useTenantPath();
  const conversationQuestions = useUsageOfConversationQuestions(
    useUsageRange(),
    conversationId,
  );
  const handleBack = useCallback(
    () =>
      void navigate({
        pathname: toTenantPath(
          APP_ROUTES.SETTINGS_ROUTES.AI_USAGE_BY_CONVERSATION,
        ),
        search: location.search,
      }),
    [navigate, toTenantPath, location.search],
  );

  return (
    <Flex direction="column">
      {conversationQuestions.error && (
        <UsageFailure failure={conversationQuestions.error} />
      )}
      {conversationQuestions.isPending && <UsageLoading />}
      {conversationQuestions.data && (
        <ConversationQuestions
          conversationQuestions={conversationQuestions.data}
          onBack={handleBack}
        />
      )}
    </Flex>
  );
}
