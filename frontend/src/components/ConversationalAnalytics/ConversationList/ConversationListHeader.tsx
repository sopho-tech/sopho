import {
  Button,
  ExpandableSearch,
  Flex,
  Heading,
  IconButton,
  Text,
} from "src/components/design-system";

type ConversationListHeaderProps = {
  search: string;
  onSearchChange: (value: string) => void;
  isSelectMode: boolean;
  selectedCount: number;
  onEnterSelectMode: () => void;
  onExitSelectMode: () => void;
  onSelectAll: () => void;
  onDeleteSelected: () => void;
  onCreateConversation: () => void;
};

export const ConversationListHeader = ({
  search,
  onSearchChange,
  isSelectMode,
  selectedCount,
  onEnterSelectMode,
  onExitSelectMode,
  onSelectAll,
  onDeleteSelected,
  onCreateConversation,
}: ConversationListHeaderProps) => (
  <Flex
    alignItems="center"
    justifyContent="space-between"
    gap="md"
    paddingY="lg"
    paddingX="2xs"
  >
    <Heading accessbilityLevel={2} size="xl" weight="semibold">
      Conversations
    </Heading>

    {isSelectMode ? (
      <Flex alignItems="center" gap="sm">
        <Text color="subtle" fontSize="sm">
          {selectedCount} selected
        </Text>
        <Button
          label="Select All"
          shape="rectangle"
          size="sm"
          backgroundColor="white"
          onClick={onSelectAll}
        />
        <Button
          label="Delete"
          shape="rectangle"
          size="sm"
          backgroundColor="red"
          disabled={selectedCount === 0}
          onClick={onDeleteSelected}
        />
        <IconButton
          type="close"
          backgroundColor="transparent"
          iconColor="grey"
          size="sm"
          tooltip={{ text: "Cancel selection" }}
          onClick={onExitSelectMode}
        />
      </Flex>
    ) : (
      <Flex alignItems="center" gap="sm">
        <ExpandableSearch
          value={search}
          onChange={onSearchChange}
          placeholder="Search conversations"
          tooltip="Search conversations"
        />
        <IconButton
          type="add"
          backgroundColor="transparent"
          iconColor="grey"
          size="sm"
          tooltip={{ text: "New conversation" }}
          onClick={onCreateConversation}
          />
          <Button
            label="Select"
            shape="rectangle"
            size="sm"
            backgroundColor="accent"
            emphasis="primary"
            onClick={onEnterSelectMode}
          />
      </Flex>
    )}
  </Flex>
);
