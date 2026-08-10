import { Flex } from "src/components/design-system/Flex";
import { IconButton } from "src/components/design-system/IconButton";
import {
  IconColor,
  IconSize,
  IconType,
  SpacingSize,
} from "src/components/design-system/datatypes";
import styles from "src/components/design-system/SplitIconButton/SplitIconButton.module.css";

export type SplitIconButtonSize = "sm" | "md";

const SIZE_VARIANTS: Record<
  SplitIconButtonSize,
  {
    gap: SpacingSize;
    paddingX: SpacingSize;
    paddingY?: SpacingSize;
    caretSize: IconSize;
  }
> = {
  sm: { gap: "2xs", paddingX: "xs", paddingY: "2xs", caretSize: "sm" },
  md: { gap: "sm", paddingX: "sm", caretSize: "sm" },
};

export type SplitIconButtonProps = {
  type: IconType;
  iconColor: IconColor;
  iconSize?: IconSize;
  size?: SplitIconButtonSize;
  onClick: () => void;
  busy?: boolean;
  primaryLabel: string;
  onSecondaryClick: () => void;
  secondaryLabel: string;
  ref?: React.Ref<HTMLDivElement>;
} & Omit<React.HTMLAttributes<HTMLDivElement>, "onClick" | "color">;

export function SplitIconButton({
  type,
  iconColor,
  iconSize,
  size = "md",
  onClick,
  busy = false,
  primaryLabel,
  onSecondaryClick,
  secondaryLabel,
  ref,
  ...triggerProps
}: SplitIconButtonProps) {
  const variant = SIZE_VARIANTS[size];

  return (
    <Flex
      ref={ref}
      {...triggerProps}
      direction="row"
      alignItems="center"
      gap={variant.gap}
      paddingX={variant.paddingX}
      paddingY={variant.paddingY}
      backgroundColor="grey"
      className={styles.group}
    >
      <IconButton
        type={type}
        backgroundColor="transparent"
        iconColor={iconColor}
        iconSize={iconSize}
        onClick={onClick}
        busy={busy}
        aria-label={primaryLabel}
      />
      <IconButton
        type="chevron_down"
        backgroundColor="transparent"
        iconColor={iconColor}
        iconSize={variant.caretSize}
        onClick={onSecondaryClick}
        className={styles.caret}
        aria-label={secondaryLabel}
      />
    </Flex>
  );
}

SplitIconButton.displayName = "SplitIconButton";
