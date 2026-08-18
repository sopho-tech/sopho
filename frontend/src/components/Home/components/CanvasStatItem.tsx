import { memo } from "react";
import { Flex, Badge, Icon, ToolTip } from "src/components/design-system";
import { IconType } from "src/components/design-system/datatypes";

type CanvasStatItemProps = {
  iconType: IconType;
  count: number;
  tooltipText: string;
};

function CanvasStatItemComponent({
  iconType,
  count,
  tooltipText,
}: CanvasStatItemProps) {
  return (
    <ToolTip messageElement={tooltipText} tooltipSide="top">
      <Flex gap="xs" alignItems="center">
        <Icon type={iconType} color="grey" />
        <Badge variant="subtle">{count}</Badge>
      </Flex>
    </ToolTip>
  );
}

export const CanvasStatItem = memo(CanvasStatItemComponent);
