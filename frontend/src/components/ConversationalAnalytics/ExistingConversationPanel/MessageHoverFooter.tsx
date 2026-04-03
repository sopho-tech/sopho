import { Flex, IconButton, Text } from "src/components/design-system";
import { formatTimestamp } from "src/utils/timestamp_utils";

type MessageHoverFooterProps = {
  createdAt: string;
  textToCopy: string;
  footerClassName: string;
};

export function MessageHoverFooter({
  createdAt,
  textToCopy,
  footerClassName,
}: MessageHoverFooterProps) {
  return (
    <Flex
      direction="row"
      alignItems="center"
      justifyContent="space-between"
      width="100%"
      paddingTop="xs"
      className={footerClassName}
    >
      <Text as="span" color="subtle" fontSize="xs">
        {formatTimestamp(createdAt) ?? ""}
      </Text>
      <Flex alignItems="center" justifyContent="flex-end" flex="none">
        <IconButton
          type="copy"
          backgroundColor="transparent"
          iconColor="grey"
          size="none"
          iconSize="sm"
          tooltip={{ text: "Copy", direction: "top" }}
          onClick={() => void navigator.clipboard.writeText(textToCopy)}
        />
      </Flex>
    </Flex>
  );
}
