import { ColumnConfig } from "src/components/Table";
import { Flex, Icon, Text } from "src/components/design-system";
import { ConversationListItemDto } from "src/api/conversational_analytics";
import { ConversationRowActions } from "src/components/ConversationalAnalytics/ConversationList/ConversationRowActions";

type CreateColumnsParams = {
  isSelectMode: boolean;
  selectedIds: Set<string>;
  onEnterSelectMode: (conversationId: string) => void;
};

export function createConversationListColumns({
  isSelectMode,
  selectedIds,
  onEnterSelectMode,
}: CreateColumnsParams): ColumnConfig<ConversationListItemDto>[] {
  return [
    {
      key: "icon",
      header: "",
      type: "display",
      size: 24,
      cell: (props) => {
        const isSelected = selectedIds.has(props.row.original.id);
        return (
          <Flex alignItems="center">
            <Icon
              type={
                isSelectMode
                  ? isSelected
                    ? "square_check"
                    : "square"
                  : "message"
              }
              color={isSelectMode && isSelected ? "accent" : "grey"}
              size="md"
            />
          </Flex>
        );
      },
    },
    {
      key: "name",
      header: "",
      type: "accessor",
      accessor: "name",
      fill: true,
      cell: (props) => (
        <Flex overflow="hidden">
          <Text truncate color="darkGrey" fontSize="sm">
            {props.getValue() as string}
          </Text>
        </Flex>
      ),
    },
    {
      key: "user_message_count",
      header: "",
      type: "accessor",
      accessor: "user_message_count",
      size: 120,
      cell: (props) => {
        const count = props.getValue() as number;
        return (
          <Flex
            justifyContent="flex-end"
            paddingRight="sm"
            sx={{ whiteSpace: "nowrap" }}
          >
            <Text color="subtle" fontSize="xs">
              {count} {count === 1 ? "message" : "messages"}
            </Text>
          </Flex>
        );
      },
    },
    {
      key: "actions",
      header: "",
      type: "display",
      size: 130,
      cell: (props) => (
        <ConversationRowActions
          conversation={props.row.original}
          isSelectMode={isSelectMode}
          onEnterSelectMode={onEnterSelectMode}
        />
      ),
    },
  ];
}
