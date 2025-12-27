import { Flex, Icon } from "src/components/design-system";
import { Text } from "src/components/design-system";
import styles from "./BannerSlim.module.css";

type BannerSlimProps = {
  type: "error" | "info";
  message: string | undefined;
};

export function BannerSlim({ type: _type, message }: BannerSlimProps) {
  if (!message) {
    return null;
  }
  return (
    <Flex
      backgroundColor="error"
      borderRadius="lg"
      paddingX="xs"
      paddingY="xs"
      alignItems="center"
      alignContent="center"
      gap="xs"
    >
      <span className={styles.iconWrapper}>
        <Icon type="triangle_alert" color="error"></Icon>
      </span>
      <Text color="default" fontSize="sm">
        {message}
      </Text>
    </Flex>
  );
}
