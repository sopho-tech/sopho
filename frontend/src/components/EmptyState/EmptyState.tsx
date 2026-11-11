import {
  Button,
  Flex,
  Heading,
  Icon,
  Text,
} from "src/components/design-system";
import { IconType } from "src/components/design-system/datatypes";

type EmptyStateAction =
  | { buttonLabel: string; onButtonClick: () => void }
  | { buttonLabel?: never; onButtonClick?: never };

type EmptyStateProps = {
  icon: IconType;
  heading: string;
  description: string;
} & EmptyStateAction;

export function EmptyState({
  icon,
  heading,
  description,
  buttonLabel,
  onButtonClick,
}: EmptyStateProps) {
  return (
    <Flex
      as="section"
      direction="column"
      gap="md"
      alignItems="center"
      alignSelf="center"
      paddingY="lg"
      paddingX="lg"
      height="100%"
      width="100%"
    >
      <Icon type={icon} color="lightgrey" size="2xl" interactive={false} />
      <Flex
        direction="column"
        gap="xs"
        alignItems="center"
        sx={{ textAlign: "center" }}
      >
        <Heading accessibilityLevel={2} textColor="default">
          {heading}
        </Heading>
        <Text as="p" color="subtle">
          {description}
        </Text>
      </Flex>
      {buttonLabel !== undefined && onButtonClick !== undefined && (
        <Button
          backgroundColor="accent"
          label={buttonLabel}
          onClick={onButtonClick}
          shape="rectangle"
          size="md"
        />
      )}
    </Flex>
  );
}
