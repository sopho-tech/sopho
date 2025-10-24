import * as Tooltip from "@radix-ui/react-tooltip";
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
    <Tooltip.Provider>
      <Tooltip.Root
        open={open}
        defaultOpen={defaultOpen}
        onOpenChange={onOpenChange}
      >
        <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
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
