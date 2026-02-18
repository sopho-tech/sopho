import {
  Button,
  Flex,
  Heading,
  Icon,
  Text,
} from "src/components/design-system";
import { IconType } from "src/components/design-system/datatypes";

type EmptyStateProps = {
  icon: IconType;
  heading: string;
  description: string;
  buttonLabel: string;
  onButtonClick: () => void;
};

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
    >
      <Icon type={icon} color="lightgrey" size="2xl" interactive={false} />
      <Flex
        direction="column"
        gap="xs"
        alignItems="center"
        sx={{ textAlign: "center" }}
      >
        <Heading accessbilityLevel={2} textColor="default">
          {heading}
        </Heading>
        <Text as="p" color="subtle">
          {description}
        </Text>
      </Flex>
      <Button
        backgroundColor="accent"
        label={buttonLabel}
        onClick={onButtonClick}
        shape="rectangle"
        size="md"
      />
    </Flex>
  );
}
