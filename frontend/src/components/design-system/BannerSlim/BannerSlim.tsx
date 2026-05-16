import type { CSSProperties, ComponentProps } from "react";
import { Flex, Icon } from "src/components/design-system";
import { Text } from "src/components/design-system/Text";
import type {
  ColorVariant,
  IconColor,
  IconType,
} from "src/components/design-system/datatypes";
import styles from "./BannerSlim.module.css";

type BannerTextColor = NonNullable<ComponentProps<typeof Text>["color"]>;

type BannerSlimProps = {
  type: "error" | "info" | "warning" | "success";
  message: string | undefined;
};

const typeConfig: Record<
  BannerSlimProps["type"],
  {
    backgroundColor: ColorVariant;
    sx?: CSSProperties;
    icon: IconType;
    iconColor: IconColor;
    textColor: BannerTextColor;
  }
> = {
  error: {
    backgroundColor: "error",
    icon: "triangle_alert",
    iconColor: "default",
    textColor: "default",
  },
  info: {
    backgroundColor: "grey",
    icon: "info",
    iconColor: "default",
    textColor: "default",
  },
  warning: {
    backgroundColor: "default",
    sx: { backgroundColor: "var(--color-yellow-200)" },
    icon: "triangle_alert",
    iconColor: "black",
    textColor: "warning",
  },
  success: {
    backgroundColor: "default",
    sx: { backgroundColor: "var(--color-green-200)" },
    icon: "circle_check",
    iconColor: "black",
    textColor: "success",
  },
};

export function BannerSlim({ type, message }: BannerSlimProps) {
  if (!message) {
    return null;
  }
  const cfg = typeConfig[type];
  return (
    <Flex
      data-type={type}
      backgroundColor={cfg.backgroundColor}
      borderRadius="lg"
      paddingX="xs"
      paddingY="xs"
      alignItems="center"
      alignContent="center"
      gap="xs"
      sx={cfg.sx}
    >
      <span className={styles.iconWrapper}>
        <Icon type={cfg.icon} color={cfg.iconColor} interactive={false} />
      </span>
      <Text color={cfg.textColor} fontSize="sm">
        {message}
      </Text>
    </Flex>
  );
}
