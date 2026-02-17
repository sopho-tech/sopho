import { Flex, Icon } from "src/components/design-system";
import { Text } from "src/components/design-system";
import styles from "./BannerSlim.module.css";

type BannerSlimProps = {
  type: "error" | "info";
  message: string | undefined;
};

export function BannerSlim({ type, message }: BannerSlimProps) {
  if (!message) {
    return null;
  }
  return (
    <Flex
      data-type={type}
      backgroundColor="error"
      borderRadius="lg"
      paddingX="xs"
      paddingY="xs"
      alignItems="center"
      alignContent="center"
      gap="xs"
    >
      <span className={styles.iconWrapper}>
        <Icon type="triangle_alert" color="default"></Icon>
      </span>
      <Text color="default" fontSize="sm">
        {message}
      </Text>
    </Flex>
  );
}
