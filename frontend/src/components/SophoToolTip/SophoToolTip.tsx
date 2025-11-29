import * as Tooltip from "@radix-ui/react-tooltip";
import React from "react";
import SophoToolTipStyles from "src/components/SophoToolTip/SophoToolTip.module.css";

type SophoToolTipProps = {
  messageElement: React.ReactNode;
  children: React.ReactNode;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (value: boolean) => void;
  tooltipSide?: "top" | "right" | "bottom" | "left";
};

export function SophoToolTip({
  messageElement,
  children,
  open,
  defaultOpen,
  onOpenChange,
  tooltipSide = "top",
}: SophoToolTipProps) {
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
                `${SophoToolTipStyles.tooltipTrigger} ${(children as React.ReactElement<any>).props?.className || ""}`.trim(),
            })
          ) : (
            <span className={SophoToolTipStyles.tooltipTrigger}>
              {children}
            </span>
          )}
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            className={SophoToolTipStyles.tooltipContent}
            sideOffset={5}
            side={tooltipSide}
            asChild
          >
            <div>
              {messageElement}
              <Tooltip.Arrow className={SophoToolTipStyles.tooltipArrow} />
            </div>
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}
