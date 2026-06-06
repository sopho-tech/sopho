import { Flex, Text } from "src/components/design-system";
import styles from "./SuggestedQuestionList.module.css";

type Question = { id: string; text: string };

type Props = {
  questions: Question[];
  onPick: (text: string) => void;
};

export function SuggestedQuestionList({ questions, onPick }: Props) {
  if (!questions.length) return null;

  return (
    <Flex
      as="ul"
      direction="column"
      gap="xs"
      sx={{ listStyle: "none", margin: 0, padding: 0 }}
    >
      {questions.map(({ id, text }) => (
        <Flex
          as="li"
          key={id}
          paddingX="md"
          paddingY="sm"
          borderRadius="lg"
          alignItems="center"
          onClick={() => onPick(text)}
          className={styles.item}
          sx={{
            cursor: "pointer",
            border:
              "var(--border-width-medium) dashed var(--border-color-light)",
          }}
        >
          <Text as="span" color="subtle" fontSize="sm">
            {text}
          </Text>
        </Flex>
      ))}
    </Flex>
  );
}
