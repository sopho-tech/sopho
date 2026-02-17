import * as Tooltip from "@radix-ui/react-tooltip";
import React from "react";
import ToolTipStyles from "src/components/design-system/ToolTip/ToolTip.module.css";

export type ToolTipProps = {
  messageElement: React.ReactNode;
  children: React.ReactNode;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (value: boolean) => void;
  tooltipSide?: "top" | "right" | "bottom" | "left";
};

export function ToolTip({
  messageElement,
  children,
  open,
  defaultOpen,
  onOpenChange,
  tooltipSide = "top",
}: ToolTipProps) {
  return (
    <Tooltip.Provider delayDuration={100}>
      <Tooltip.Root
        open={open}
        defaultOpen={defaultOpen}
        onOpenChange={onOpenChange}
      >
        <Tooltip.Trigger asChild>
          {React.isValidElement(children) ? (
            React.cloneElement(children as React.ReactElement<any>, {
              className:
                `${ToolTipStyles.tooltipTrigger} ${(children as React.ReactElement<any>).props?.className || ""}`.trim(),
            })
          ) : (
            <span className={ToolTipStyles.tooltipTrigger}>{children}</span>
          )}
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            className={ToolTipStyles.tooltipContent}
            sideOffset={5}
            side={tooltipSide}
            asChild
          >
            <div>
              {messageElement}
              {/* <Tooltip.Arrow className={ToolTipStyles.tooltipArrow} /> */}
            </div>
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}
