import { PlatformAiAllowance } from "src/api/ai_configuration";
import { BannerSlim, Flex, Heading, Text } from "src/components/design-system";
import { formatWholeNumber } from "src/utils/number_format";

type PlatformAiCardProps = {
  platformAi: PlatformAiAllowance;
  unavailableMessage: string | null;
};

export function PlatformAiCard({
  platformAi,
  unavailableMessage,
}: PlatformAiCardProps) {
  return (
    <Flex
      as="section"
      direction="column"
      gap="sm"
      paddingX="md"
      paddingY="md"
      border="default"
      borderRadius="lg"
      aria-label="Platform AI"
    >
      <Heading accessibilityLevel={3} size="lg" textColor="black">
        Platform AI
      </Heading>
      <Text fontSize="sm" color="subtle">
        This tenant has no AI key of its own, so AI features run on the
        platform&apos;s key within a token allowance. Add a key of your own
        below to remove the limit.
      </Text>
      <Flex direction="row" gap="lg" flexWrap="wrap">
        <Flex direction="column" gap="2xs">
          <Text fontSize="sm" color="subtle">
            Tokens used
          </Text>
          <Text fontSize="lg" color="black">
            {`${formatWholeNumber(platformAi.used_tokens)} of ${formatWholeNumber(platformAi.token_allowance)}`}
          </Text>
        </Flex>
        <Flex direction="column" gap="2xs">
          <Text fontSize="sm" color="subtle">
            Tokens left
          </Text>
          <Text fontSize="lg" color="black">
            {formatWholeNumber(platformAi.remaining_tokens)}
          </Text>
        </Flex>
      </Flex>
      {unavailableMessage && (
        <BannerSlim type="warning" message={unavailableMessage} />
      )}
    </Flex>
  );
}
