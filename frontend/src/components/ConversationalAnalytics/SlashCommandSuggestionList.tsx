import { Badge, Flex, Text } from "src/components/design-system";
import type { SlashCommand } from "src/constants/slash_commands";
import styles from "./SlashCommandSuggestionList.module.css";

type SlashCommandSuggestionListProps = {
  commands: SlashCommand[];
  onPick: (commandName: string) => void;
};

export function SlashCommandSuggestionList({
  commands,
  onPick,
}: SlashCommandSuggestionListProps) {
  if (!commands.length) return null;

  return (
    <Flex
      as="ul"
      gap="xs"
      alignItems="center"
      sx={{ listStyle: "none", margin: 0, padding: 0, flexWrap: "wrap" }}
    >
      {commands.map(({ name, description }) => (
        <Flex
          as="li"
          key={name}
          gap="xs"
          paddingX="md"
          paddingY="sm"
          borderRadius="lg"
          alignItems="center"
          onClick={() => onPick(name)}
          className={styles.item}
          sx={{
            cursor: "pointer",
            border:
              "var(--border-width-medium) dashed var(--border-color-light)",
          }}
        >
          <Badge variant="command" shape="rounded" size="sm">
            {`/${name}`}
          </Badge>
          <Text as="span" color="subtle" fontSize="sm">
            {description}
          </Text>
        </Flex>
      ))}
    </Flex>
  );
}
